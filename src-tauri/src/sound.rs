pub const SOUND_NAMES: &[&str] = &[
    "Glass", "Ping", "Pop", "Hero", "Tink", "Basso", "Blow", "Bottle", "Funk", "Morse",
];

#[cfg(target_os = "macos")]
pub fn play_sound(name: &str) {
    use objc2_app_kit::NSSound;
    use objc2_foundation::NSString;

    let ns_name = NSString::from_str(name);
    if let Some(sound) = NSSound::soundNamed(&ns_name) {
        sound.play();
    }
}
