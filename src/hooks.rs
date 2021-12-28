use penrose::{
    core::{
        hooks::Hook,
        xconnection::{XConn},
        manager::WindowManager
    }
};

pub struct StartupScript {
    path: String,
}

impl StartupScript {
    pub fn new(s: impl Into<String>) -> Self {
        Self { path: s.into() }
    }
}

impl <X: XConn> Hook<X> for StartupScript {
    fn startup(&mut self, _: &mut WindowManager<X>) -> penrose::Result<()> {
        spawn!(&self.path)
    }
}


