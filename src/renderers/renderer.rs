use crate::{camera::Camera, scene::Scene};

pub trait Renderer {
    type Error;
    type Canvas<'a>;

    fn render(
        &mut self,
        canvas: &mut Self::Canvas<'_>,
        scene: &Scene,
        camera: &Camera,
    ) -> Result<(), Self::Error>;
}
