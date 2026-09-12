// A deterministic, low-amplitude idle gaze loop for a resolved pair of eye
// bone transforms. It deliberately owns direction only; an AVC eye tracker
// may remain attached with `.enable_pupil_direction_tracking(false)` so its
// left/right closure samples continue to drive blink morphs.
//
// `beats_per_second` converts the authored seconds below to the scene clock's
// beat unit: pass `Clock BPM / 60` (for example, `1.5` for a 90 BPM clock).
// Both transform parameters must be the actual mapped skin-joint transforms.
// They may carry any bind pose: `rest_relative_rotation` preserves their
// immutable GLTF rest translation, rotation, and scale.

export fn ambient_eye_saccades(left_eye, right_eye, beats_per_second) {
    let transition_duration = 0.16 * beats_per_second
    let left_transition = Transition {
        duration_beats(transition_duration)
        ease_in_out_sine()
        replace_same_target()
    }
    let right_transition = Transition {
        duration_beats(transition_duration)
        ease_in_out_sine()
        replace_same_target()
    }
    left_eye.attach(left_transition)
    right_eye.attach(right_transition)

    let pose = fn(pitch, yaw, convergence) {
        // Positive pitch is up; yaw is deliberately kept below 0.10 rad.
        // The small convergence value prevents a visibly perfect stereo lock.
        left_eye.rest_relative_rotation([pitch, yaw + convergence, 0.0])
        right_eye.rest_relative_rotation([pitch * 0.96, yaw - convergence, 0.0])
    }

    return Animation.looping().length(32.10 * beats_per_second) {
        // 32 irregular samples. Every adjacent interval, including the
        // 1.24-second wrap from the final sample to this first one, is within
        // 300–2000 ms.
        Keyframe.at(0.00 * beats_per_second) { pose( 0.008, -0.018, 0.003) }
        Keyframe.at(0.72 * beats_per_second) { pose( 0.021,  0.034, 0.004) }
        Keyframe.at(1.29 * beats_per_second) { pose( 0.006,  0.012, 0.002) }
        Keyframe.at(2.77 * beats_per_second) { pose(-0.028, -0.052, 0.004) }
        Keyframe.at(3.21 * beats_per_second) { pose(-0.023, -0.047, 0.003) }
        Keyframe.at(4.11 * beats_per_second) { pose( 0.015, -0.004, 0.002) }
        Keyframe.at(5.91 * beats_per_second) { pose( 0.043,  0.061, 0.005) }
        Keyframe.at(6.27 * beats_per_second) { pose( 0.039,  0.057, 0.004) }
        Keyframe.at(7.42 * beats_per_second) { pose(-0.004,  0.026, 0.003) }
        Keyframe.at(8.08 * beats_per_second) { pose(-0.016, -0.014, 0.002) }
        Keyframe.at(9.67 * beats_per_second) { pose( 0.032, -0.074, 0.005) }
        Keyframe.at(10.19 * beats_per_second) { pose( 0.027, -0.069, 0.004) }
        Keyframe.at(11.21 * beats_per_second) { pose( 0.003, -0.008, 0.002) }
        Keyframe.at(12.95 * beats_per_second) { pose(-0.047,  0.018, 0.004) }
        Keyframe.at(13.33 * beats_per_second) { pose(-0.043,  0.015, 0.003) }
        Keyframe.at(14.08 * beats_per_second) { pose(-0.010,  0.048, 0.004) }
        Keyframe.at(15.36 * beats_per_second) { pose( 0.018,  0.079, 0.005) }
        Keyframe.at(16.08 * beats_per_second) { pose( 0.014,  0.073, 0.004) }
        Keyframe.at(17.93 * beats_per_second) { pose(-0.019,  0.007, 0.002) }
        Keyframe.at(18.40 * beats_per_second) { pose(-0.014,  0.004, 0.002) }
        Keyframe.at(19.38 * beats_per_second) { pose( 0.051, -0.031, 0.004) }
        Keyframe.at(20.88 * beats_per_second) { pose( 0.046, -0.027, 0.004) }
        Keyframe.at(21.29 * beats_per_second) { pose( 0.017, -0.011, 0.003) }
        Keyframe.at(22.35 * beats_per_second) { pose(-0.031, -0.064, 0.005) }
        Keyframe.at(23.02 * beats_per_second) { pose(-0.027, -0.058, 0.004) }
        Keyframe.at(24.98 * beats_per_second) { pose( 0.012,  0.006, 0.002) }
        Keyframe.at(25.55 * beats_per_second) { pose( 0.038,  0.052, 0.004) }
        Keyframe.at(26.64 * beats_per_second) { pose( 0.034,  0.047, 0.004) }
        Keyframe.at(27.42 * beats_per_second) { pose(-0.006,  0.020, 0.003) }
        Keyframe.at(29.07 * beats_per_second) { pose(-0.036, -0.036, 0.004) }
        Keyframe.at(29.44 * beats_per_second) { pose(-0.027, -0.029, 0.003) }
        Keyframe.at(30.86 * beats_per_second) { pose( 0.010, -0.022, 0.003) }
    }
}
