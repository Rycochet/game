use super::*;

#[test]
fn a_spinner_gets_its_picture() {
    let mut app = App::new();
    app.init_resource::<Assets<Image>>()
        .add_systems(Startup, make_spinner_picture)
        .add_systems(Update, picture_spinners);
    let spinner = app.world_mut().spawn((Node::default(), Spinner)).id();
    app.update();
    let picture = app.world().resource::<SpinnerPicture>().0.clone();
    let image = app.world().get::<ImageNode>(spinner).unwrap();
    assert_eq!(image.image, picture);
}
