pub const APP_ICON_PNG: &[u8] = include_bytes!(
    concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/images/clover_viewer.png"
    )
);

pub const FONT_MSYHL: &[u8] = include_bytes!(
    concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/fonts/msyhl.ttf"
    )
);
