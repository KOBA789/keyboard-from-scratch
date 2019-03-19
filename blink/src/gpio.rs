#[allow(dead_code)]
pub enum Mode {
    Input,
    Output10MHz,
    Output2MHz,
    Output50MHz,
}
impl Mode {
    pub fn bits(&self) -> u8 {
        use Mode::*;
        match self {
            Input       => 0b00,
            Output10MHz => 0b01,
            Output2MHz  => 0b10,
            Output50MHz => 0b11,
        }
    }
}

#[allow(dead_code)]
pub enum InputCnf {
    Analog,
    Float,
    PullUpdown,
}
impl InputCnf {
    pub fn bits(&self) -> u8 {
        use InputCnf::*;
        match self {
            Analog     => 0b00,
            Float      => 0b01,
            PullUpdown => 0b10,
        }
    }
}

#[allow(dead_code)]
pub enum OutputCnf {
    Pushpull,
    Opendrain,
    AltfnPushpull,
    AltfnOpendrain,
}
impl OutputCnf {
    pub fn bits(&self) -> u8 {
        use OutputCnf::*;
        match self {
            Pushpull       => 0b00,
            Opendrain      => 0b01,
            AltfnPushpull  => 0b10,
            AltfnOpendrain => 0b11,
        }
    }
}
