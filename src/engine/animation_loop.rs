pub struct AnimationLoop<'a> {
    universe: &'a mut crate::engine::Universe,
    renderer: &'a mut crate::engine::graphics::Renderer,
}

impl<'a> AnimationLoop<'a> {
    pub fn new(
        universe: &'a mut crate::engine::Universe,
        renderer: &'a mut crate::engine::graphics::Renderer,
    ) -> EngineResult<Self> {
        Ok(Self { universe, renderer })
    }

    pub fn start(&mut self) -> EngineResult<()> {
        self.universe.sync_visuals();
        self.universe.render(self.renderer);
        Ok(())
    }
}
