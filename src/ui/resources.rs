pub const APP_IMG: &[u8] = include_bytes!(
    concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/images/clover_viewer.png"
    )
);
pub const APP_FONT: &[u8] = include_bytes!(
    concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/fonts/msyhl.ttf"
    )
);
