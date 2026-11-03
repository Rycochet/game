use super::*;

#[test]
fn each_layer_is_smaller_and_in_front_of_the_last() {
    assert!(layer_share(1) < layer_share(0));
    assert!((layer_share(0) - 0.9).abs() < 0.001);
    assert!(layer_z(1) > layer_z(0));
}
