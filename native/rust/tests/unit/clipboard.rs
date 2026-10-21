use super::*;

#[test]
fn validates_clipboard_dimensions_before_accepting_pixels() {
    assert!(decode_image("[1,1]", vec![10, 20, 30, 40]).is_ok());
    for metadata in [
        "[0,1]",
        "[1,0]",
        "[4294967295,4294967295]",
        "[-1,1]",
        "[1]",
        "{}",
    ] {
        assert!(decode_image(metadata, vec![0; 4]).is_err(), "{metadata}");
    }
    assert!(decode_image("[2,1]", vec![0; 4]).is_err());
    assert!(decode_image("[1,1]", vec![0; 8]).is_err());
    assert!(image_byte_len(8192, 8192).is_err());
}

#[test]
fn hardware_copy_exports_selection_and_external_paste_adds_a_layer() {
    use egui::{Key, Modifiers};
    use photocraft_ui_egui::{PhotocraftApp, Services};
    use std::sync::{Arc, Mutex};

    type Image = Option<(u32, u32, Vec<u8>)>;
    let os: Arc<Mutex<Image>> = Arc::default();
    let writes = Arc::new(Mutex::new(0));
    let (out, incoming, count) = (os.clone(), os.clone(), writes.clone());
    let mut app = PhotocraftApp::new(
        photocraft_engine::Session::new(),
        Services {
            clipboard_set_image: Some(Box::new(move |w, h, pixels| {
                *out.lock().unwrap() = Some((w, h, pixels.to_vec()));
                *count.lock().unwrap() += 1;
                Ok(())
            })),
            clipboard_get_image: Some(Box::new(move || incoming.lock().unwrap().clone())),
            ..Default::default()
        },
    );
    app.run("file.new", serde_json::json!({"width": 64, "height": 64}))
        .unwrap();
    let ctx = egui::Context::default();
    let frame = |app: &mut PhotocraftApp, events: Vec<egui::Event>| {
        let mut raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280., 800.),
            )),
            events,
            ..Default::default()
        };
        app.host_input(&ctx, &mut raw);
        crate::editor_frame(&ctx, app, raw).drop_without_applying_deltas();
    };
    for _ in 0..3 {
        frame(&mut app, Vec::new());
    }
    app.run(
        "select.rect",
        serde_json::json!({"x": 2, "y": 3, "width": 8, "height": 4}),
    )
    .unwrap();
    let modifiers = Modifiers::COMMAND;
    frame(
        &mut app,
        vec![egui::Event::Key {
            key: Key::C,
            physical_key: Some(Key::C),
            pressed: true,
            repeat: false,
            modifiers,
        }],
    );
    let (width, height, pixels) = os.lock().unwrap().clone().unwrap();
    assert_eq!((width, height, pixels.len()), (8, 4, 128));
    frame(
        &mut app,
        vec![egui::Event::Key {
            key: Key::C,
            physical_key: Some(Key::C),
            pressed: false,
            repeat: false,
            modifiers,
        }],
    );
    frame(&mut app, Vec::new());
    assert_eq!(
        *writes.lock().unwrap(),
        1,
        "release and idle frames must not copy again"
    );

    *os.lock().unwrap() = Some((3, 2, [255, 0, 0, 255].repeat(6)));
    let layers = app.session.active().unwrap().doc.layers.len();
    frame(
        &mut app,
        vec![egui::Event::Key {
            key: Key::V,
            physical_key: Some(Key::V),
            pressed: true,
            repeat: false,
            modifiers,
        }],
    );
    assert_eq!(app.session.active().unwrap().doc.layers.len(), layers + 1);
    let state = app.session.active().unwrap();
    let surface = state
        .doc
        .layer(state.active_layer.unwrap())
        .unwrap()
        .surface()
        .unwrap();
    assert_eq!(
        (
            surface.content_bounds().width(),
            surface.content_bounds().height()
        ),
        (3, 2)
    );
    frame(
        &mut app,
        vec![egui::Event::Key {
            key: Key::V,
            physical_key: Some(Key::V),
            pressed: false,
            repeat: false,
            modifiers,
        }],
    );
    assert_eq!(
        app.session.active().unwrap().doc.layers.len(),
        layers + 1,
        "release must not paste twice"
    );
}
