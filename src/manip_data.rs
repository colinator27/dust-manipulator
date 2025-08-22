pub struct ManipSetup {
    pub text: &'static str,
    pub rng_amount: usize   
}
impl ManipSetup {
    const fn new(text: &'static str, rng_amount: usize) -> Self {
        ManipSetup {
            text, rng_amount
        }
    }
}

pub const MANIP_SETUPS_CORE: [ManipSetup; 34] = [
    ManipSetup::new("(nothing)", 0),
    ManipSetup::new("Stick - USE", 104),
    ManipSetup::new("Stick - INFO", 116),
    ManipSetup::new("Tough Glove - INFO", 164), // ManipData::new("Ballet Shoes - INFO", 164),
    ManipSetup::new("Instant Noodles - INFO", 158),
    ManipSetup::new("Stick - USE (x2)", 208),
    ManipSetup::new("Switch Armor (x2)", 114), // assumes ballet shoes and burnt pan, in either order
    ManipSetup::new("Stick - INFO (x2)", 232),
    ManipSetup::new("Tough Glove - INFO (x2)", 328), // ManipData::new("Ballet Shoes - INFO (x2)", 328),
    ManipSetup::new("Instant Noodles - INFO (x2)", 316),
    ManipSetup::new("Stick - USE\nStick - INFO", 220),
    ManipSetup::new("Stick - USE\nTough Glove - INFO", 268),
    ManipSetup::new("Stick - USE\nInstant Noodles - INFO", 262),
    ManipSetup::new("Stick - INFO\nTough Glove - INFO", 280),
    ManipSetup::new("Stick - INFO\nInstant Noodles - INFO", 274),
    ManipSetup::new("Instant Noodles - INFO\nTough Glove - INFO", 322),
    ManipSetup::new("Stick - USE (x3)", 312),
    ManipSetup::new("Stick - INFO (x3)", 348),
    ManipSetup::new("Instant Noodles - INFO (x3)", 474),
    ManipSetup::new("Tough Glove - INFO (x3)", 492),
    ManipSetup::new("Stick - USE (x2)\nStick - INFO", 324),
    ManipSetup::new("Stick - USE (x2)\nTough Glove - INFO", 372),
    ManipSetup::new("Stick - USE (x2)\nInstant Noodles - INFO", 366),
    ManipSetup::new("Instant Noodles - INFO (x2)\nStick - USE", 420),
    ManipSetup::new("Tough Glove - INFO (x2)\nStick - USE", 432),
    ManipSetup::new("Stick - INFO (x2)\nStick - USE", 336),
    ManipSetup::new("Stick - INFO (x2)\nTough Glove - INFO", 396),
    ManipSetup::new("Stick - INFO (x2)\nInstant Noodles - INFO", 390),
    ManipSetup::new("Stick - USE (x3)\nStick - INFO", 428),
    ManipSetup::new("Stick - USE (x3)\nTough Glove - INFO", 476),
    ManipSetup::new("Stick - USE (x3)\nInstant Noodles - INFO", 470),
    ManipSetup::new("Stick - INFO (x2)\nStick - USE (x2)", 440),
    ManipSetup::new("Instant Noodles - INFO (x2)\nStick - USE (x2)", 524),
    ManipSetup::new("Tough Glove - INFO (x2)\nStick - USE (x2)", 536)
];
