use super::World;

/// System trait placeholder.
pub trait System {
    fn tick(&mut self, world: &mut World);
}
