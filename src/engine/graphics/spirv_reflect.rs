use std::collections::{HashMap, HashSet};

use rspirv::binary::Parser;
use rspirv::dr::{Loader, Operand};
use rspirv::spirv::{Decoration, Op, StorageClass, Word};

fn parse_spirv(bytes: &[u8]) -> Option<rspirv::dr::Module> {
    let mut loader = Loader::new();
    Parser::new(bytes, &mut loader).parse().ok()?;
    Some(loader.module())
}

fn operand_u32(op: &Operand) -> Option<u32> {
    match op {
        Operand::LiteralBit32(v) => Some(*v),
        // rspirv represents many numeric operands as IdRef.
        Operand::IdRef(v) => Some(*v),
        _ => None,
    }
}

fn operand_decoration(op: &Operand) -> Option<Decoration> {
    match op {
        Operand::Decoration(d) => Some(*d),
        _ => None,
    }
}

fn operand_string(op: &Operand) -> Option<&str> {
    match op {
        Operand::LiteralString(s) => Some(s.as_str()),
        _ => None,
    }
}

/// Very small SPIR-V reflection targeted to one thing:
/// print push-constant block struct(s) and their computed total size.
///
/// Enable via env var LC_REFLECT_PUSH_CONSTANTS=1
pub fn print_push_constants_once() {
    if std::env::var("LC_REFLECT_PUSH_CONSTANTS").ok().as_deref() != Some("1") {
        return;
    }

    // Include the exact SPIR-V bytes that the renderer uses.
    let spv = include_bytes!("shaders/spv/unlit-mesh.vert.spv");
    let Some(module) = parse_spirv(spv) else {
        println!("[SPIRV] failed to parse unlit-mesh.vert.spv");
        return;
    };

    // Maps for quick lookup.
    let mut type_sizes: HashMap<Word, u32> = HashMap::new();
    let mut type_members: HashMap<Word, Vec<(Word, Option<u32>)>> = HashMap::new();
    let mut pointers: HashMap<Word, (StorageClass, Word)> = HashMap::new();
    let mut var_to_ptr: HashMap<Word, Word> = HashMap::new();
    let mut member_offsets: HashMap<(Word, u32), u32> = HashMap::new();
    let mut names: HashMap<Word, String> = HashMap::new();

    // Collect names + decorations.
    for inst in module.all_inst_iter() {
        match inst.class.opcode {
            Op::Name => {
                // operands: target id, literal string
                if inst.operands.len() >= 2 {
                    let id = inst.operands[0].unwrap_id_ref();
                    let s = operand_string(&inst.operands[1]).unwrap_or("<unnamed>");
                    names.insert(id, s.to_string());
                }
            }
            Op::Decorate => {
                // We don't need these for push-constant reflection; avoid unwrapping.
            }
            Op::MemberDecorate => {
                // operands: struct type id, member idx, decoration, literal
                if inst.operands.len() >= 4 {
                    let ty = inst.operands[0].unwrap_id_ref();
                    let member = operand_u32(&inst.operands[1]).unwrap_or(0);
                    let dec = operand_decoration(&inst.operands[2]);
                    let lit = operand_u32(&inst.operands[3]);
                    if dec == Some(Decoration::Offset) {
                        if let Some(lit) = lit {
                            member_offsets.insert((ty, member), lit);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // First pass: figure out scalar sizes.
    for inst in module.types_global_values.iter() {
        match inst.class.opcode {
            Op::TypeInt => {
                // operands: width, signed
                let width = operand_u32(&inst.operands[0]).unwrap_or(32);
                type_sizes.insert(inst.result_id.unwrap(), (width / 8) as u32);
            }
            Op::TypeFloat => {
                let width = operand_u32(&inst.operands[0]).unwrap_or(32);
                type_sizes.insert(inst.result_id.unwrap(), (width / 8) as u32);
            }
            _ => {}
        }
    }

    // Second pass: composite type sizes (conservative std140-ish for vec/mat).
    for inst in module.types_global_values.iter() {
        match inst.class.opcode {
            Op::TypeVector => {
                let comp_ty = inst.operands[0].unwrap_id_ref();
                let n = operand_u32(&inst.operands[1]).unwrap_or(4) as u32;
                let comp_sz = *type_sizes.get(&comp_ty).unwrap_or(&4);
                // std140: vec2 align 8, vec3/vec4 align 16. size rounds up to align.
                let align = match n {
                    1 => comp_sz,
                    2 => 8,
                    3 | 4 => 16,
                    _ => 16,
                };
                let raw = comp_sz * n;
                let size = ((raw + align - 1) / align) * align;
                type_sizes.insert(inst.result_id.unwrap(), size);
            }
            Op::TypeMatrix => {
                let col_ty = inst.operands[0].unwrap_id_ref();
                let cols = operand_u32(&inst.operands[1]).unwrap_or(4) as u32;
                let col_sz = *type_sizes.get(&col_ty).unwrap_or(&16);
                // std140: matrix is array of column vectors, each column aligned to 16.
                let stride = ((col_sz + 15) / 16) * 16;
                type_sizes.insert(inst.result_id.unwrap(), stride * cols);
            }
            Op::TypeArray => {
                // array element size * length; we ignore runtime arrays
                let elem_ty = inst.operands[0].unwrap_id_ref();
                let len_id = inst.operands[1].unwrap_id_ref();
                // length is a constant.
                let mut len: Option<u32> = None;
                for g in module.types_global_values.iter() {
                    if g.result_id == Some(len_id) && g.class.opcode == Op::Constant {
                        len = Some(operand_u32(&g.operands[0]).unwrap_or(0) as u32);
                        break;
                    }
                }
                if let Some(len) = len {
                    let elem_sz = *type_sizes.get(&elem_ty).unwrap_or(&0);
                    type_sizes.insert(inst.result_id.unwrap(), elem_sz * len);
                }
            }
            Op::TypeStruct => {
                // We'll compute using member offsets if present.
                let members: Vec<(Word, Option<u32>)> = inst
                    .operands
                    .iter()
                    .enumerate()
                    .map(|(i, op)| {
                        let ty = op.unwrap_id_ref();
                        let off = member_offsets.get(&(inst.result_id.unwrap(), i as u32)).copied();
                        (ty, off)
                    })
                    .collect();
                type_members.insert(inst.result_id.unwrap(), members);
            }
            Op::TypePointer => {
                let sc = inst.operands[0].unwrap_storage_class();
                let ty = inst.operands[1].unwrap_id_ref();
                pointers.insert(inst.result_id.unwrap(), (sc, ty));
            }
            _ => {}
        }
    }

    // Variables.
    for inst in module.types_global_values.iter() {
        if inst.class.opcode == Op::Variable {
            let ptr_ty = inst.result_type.unwrap();
            var_to_ptr.insert(inst.result_id.unwrap(), ptr_ty);
        }
    }

    // Find push-constant variables.
    let mut pc_ptr_tys: HashSet<Word> = HashSet::new();
    for (_var_id, ptr_ty) in var_to_ptr.iter() {
        if let Some((sc, _pointee)) = pointers.get(ptr_ty) {
            if *sc == StorageClass::PushConstant {
                pc_ptr_tys.insert(*ptr_ty);
            }
        }
    }

    if pc_ptr_tys.is_empty() {
        println!("[SPIRV] unlit-mesh.vert: no push constant variables found");
        return;
    }

    println!("[SPIRV] unlit-mesh.vert push constants:");
    for ptr_ty in pc_ptr_tys {
        let (_sc, pointee) = pointers[&ptr_ty];
        let ty_name = names
            .get(&pointee)
            .cloned()
            .unwrap_or_else(|| format!("id{}", pointee));

        // Compute struct size from member offsets if possible.
        let size = if let Some(members) = type_members.get(&pointee) {
            let mut max_end = 0u32;
            for (i, (m_ty, off)) in members.iter().enumerate() {
                let m_sz = *type_sizes.get(m_ty).unwrap_or(&0);
                let off = off.unwrap_or((i as u32) * m_sz);
                max_end = max_end.max(off + m_sz);
            }
            // std140 struct size typically rounded up to 16.
            ((max_end + 15) / 16) * 16
        } else {
            *type_sizes.get(&pointee).unwrap_or(&0)
        };

        println!("  - block {} (type {}): computed_size={} bytes", ty_name, pointee, size);
        if let Some(members) = type_members.get(&pointee) {
            for (i, (m_ty, off)) in members.iter().enumerate() {
                let m_name = names
                    .get(m_ty)
                    .cloned()
                    .unwrap_or_else(|| format!("type{}", m_ty));
                let m_sz = *type_sizes.get(m_ty).unwrap_or(&0);
                println!("      member[{}]: {} ({} bytes) offset={:?}", i, m_name, m_sz, off);
            }
        }
    }
}
