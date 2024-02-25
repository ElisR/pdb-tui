/// Marker trait used for managing valid state of UI
pub trait StateMarker {}

#[derive(Default, Debug, Clone, Copy)]
pub struct RenderState;

#[derive(Default, Debug, Clone, Copy)]
pub struct HelpState;

#[derive(Default, Debug, Clone, Copy)]
pub struct BenchmarkState;

impl StateMarker for HelpState {}
impl StateMarker for RenderState {}
impl StateMarker for BenchmarkState {}

#[derive(Default, Debug, Clone, Copy)]
pub struct App<S: StateMarker> {
    pub should_quit: bool,

    state: std::marker::PhantomData<S>,
}

impl From<App<HelpState>> for App<RenderState> {
    fn from(value: App<HelpState>) -> Self {
        Self {
            should_quit: value.should_quit,
            state: std::marker::PhantomData::<RenderState>,
        }
    }
}

impl From<App<RenderState>> for App<HelpState> {
    fn from(value: App<RenderState>) -> Self {
        Self {
            should_quit: value.should_quit,
            state: std::marker::PhantomData::<HelpState>,
        }
    }
}

impl From<App<BenchmarkState>> for App<RenderState> {
    fn from(value: App<BenchmarkState>) -> Self {
        Self {
            should_quit: value.should_quit,
            state: std::marker::PhantomData::<RenderState>,
        }
    }
}

impl From<App<RenderState>> for App<BenchmarkState> {
    fn from(value: App<RenderState>) -> Self {
        Self {
            should_quit: value.should_quit,
            state: std::marker::PhantomData::<BenchmarkState>,
        }
    }
}
