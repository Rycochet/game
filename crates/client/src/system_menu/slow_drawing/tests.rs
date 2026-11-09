use super::*;

fn adapter(name: &str, device_type: DeviceType) -> AdapterInfo {
    AdapterInfo {
        name: name.into(),
        ..AdapterInfo::new(device_type, wgpu_types::Backend::Gl)
    }
}

#[test]
fn a_graphics_card_is_not_slow() {
    let card = adapter("NVIDIA GeForce RTX 4060", DeviceType::DiscreteGpu);
    assert!(!is_slow(&card, false));
    let laptop = adapter(
        "ANGLE (Intel, Intel(R) Iris(R) Xe Graphics)",
        DeviceType::IntegratedGpu,
    );
    assert!(!is_slow(&laptop, false));
}

#[test]
fn software_drawing_is_slow_by_type_or_name() {
    assert!(is_slow(&adapter("anything", DeviceType::Cpu), false));
    let browser = adapter(
        "ANGLE (Google, Vulkan 1.3.0 (SwiftShader Device (Subzero)), SwiftShader driver)",
        DeviceType::Other,
    );
    assert!(is_slow(&browser, false));
    let windows = adapter("Microsoft Basic Render Driver", DeviceType::Other);
    assert!(is_slow(&windows, false));
    assert!(is_slow(
        &adapter("llvmpipe (LLVM 17.0.6)", DeviceType::Other),
        false
    ));
}

#[test]
fn the_page_can_say_so_when_the_name_is_hidden() {
    let hidden = adapter("WebKit WebGL", DeviceType::Other);
    assert!(!is_slow(&hidden, false));
    assert!(is_slow(&hidden, true));
}
