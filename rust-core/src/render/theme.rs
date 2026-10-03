use super::canvas::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeId {
    Dark = 0,
    Light = 1,
    Amoled = 2,
    Sunset = 3,
}

#[derive(Clone, Debug)]
pub struct RynkTheme {
    pub id: ThemeId,
    pub bg_color: Color,
    pub key_normal: Color,
    pub key_pressed: Color,
    pub key_modifier: Color,
    pub key_accent: Color,
    pub key_accent_text: Color,
    pub key_shadow: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub brand_accent: Color,
    pub suggestion_bar_bg: Color,
    pub suggestion_chip_bg: Color,
    pub suggestion_chip_active: Color,
    pub suggestion_text: Color,
    pub suggestion_text_active: Color,
    pub popup_bg: Color,
    pub popup_border: Color,
    pub popup_text: Color,
    pub ripple_color: Color,
    pub space_branding_color: Color,
    pub divider_color: Color,
}

impl RynkTheme {
    pub fn dark() -> Self {
        Self {
            id: ThemeId::Dark,
            bg_color: Color::rgb(42, 44, 51), // Modern Yandex Dark #2A2C33
            key_normal: Color::rgb(60, 63, 74), // Visible elevated keys #3C3F4A
            key_pressed: Color::rgb(76, 80, 94), // Tactile press #4C505E
            key_modifier: Color::rgb(50, 52, 62), // Dark modifier tint #32343E
            key_accent: Color::rgb(255, 219, 77), // Signature Yandex Warm Gold #FFDB4D
            key_accent_text: Color::rgb(26, 27, 34), // Deep text on accent
            key_shadow: Color::rgba(0, 0, 0, 40),
            text_primary: Color::rgb(255, 255, 255),
            text_secondary: Color::rgb(157, 161, 175),
            brand_accent: Color::rgb(255, 219, 77),
            suggestion_bar_bg: Color::rgb(42, 44, 51),
            suggestion_chip_bg: Color::rgb(60, 63, 74),
            suggestion_chip_active: Color::rgb(255, 219, 77),
            suggestion_text: Color::rgb(245, 246, 250),
            suggestion_text_active: Color::rgb(26, 27, 34),
            popup_bg: Color::rgb(60, 63, 74),
            popup_border: Color::rgb(80, 84, 98),
            popup_text: Color::rgb(255, 255, 255),
            ripple_color: Color::rgba(255, 219, 77, 65),
            space_branding_color: Color::rgba(157, 161, 175, 190),
            divider_color: Color::rgb(55, 58, 68),
        }
    }

    pub fn light() -> Self {
        Self {
            id: ThemeId::Light,
            bg_color: Color::rgb(238, 240, 245), // Original clean Yandex light #EEF0F5
            key_normal: Color::rgb(255, 255, 255), // Pure snow-white elevated keys #FFFFFF
            key_pressed: Color::rgb(222, 226, 235), // Soft tactile press #DEE2EB
            key_modifier: Color::rgb(214, 218, 227), // Distinct Yandex light-slate modifier #D6DAE3
            key_accent: Color::rgb(255, 219, 77), // Signature Yandex Warm Gold #FFDB4D
            key_accent_text: Color::rgb(26, 27, 34), // Sharp contrast dark on accent
            key_shadow: Color::rgba(0, 0, 0, 28), // Soft natural drop shadow
            text_primary: Color::rgb(28, 30, 36), // Crisp deep graphite text #1C1E24
            text_secondary: Color::rgb(134, 138, 150), // Secondary sub-label hints #868A96
            brand_accent: Color::rgb(255, 219, 77), // Warm signature gold
            suggestion_bar_bg: Color::rgb(238, 240, 245), // Seamless with keyboard body
            suggestion_chip_bg: Color::rgb(255, 255, 255), // Crisp white suggestion pill
            suggestion_chip_active: Color::rgb(255, 219, 77), // Active gold chip
            suggestion_text: Color::rgb(28, 30, 36),
            suggestion_text_active: Color::rgb(26, 27, 34),
            popup_bg: Color::rgb(255, 255, 255),
            popup_border: Color::rgb(218, 222, 232),
            popup_text: Color::rgb(26, 27, 34),
            ripple_color: Color::rgba(255, 219, 77, 75),
            space_branding_color: Color::rgba(134, 138, 150, 210),
            divider_color: Color::rgb(224, 227, 236),
        }
    }

    pub fn amoled() -> Self {
        Self {
            id: ThemeId::Amoled,
            bg_color: Color::rgb(0, 0, 0), // Pitch black
            key_normal: Color::rgb(18, 18, 18),
            key_pressed: Color::rgb(35, 35, 35),
            key_modifier: Color::rgb(12, 12, 12),
            key_accent: Color::rgb(0, 245, 212), // Electric Neon
            key_accent_text: Color::rgb(0, 0, 0),
            key_shadow: Color::rgba(0, 0, 0, 0),
            text_primary: Color::rgb(255, 255, 255),
            text_secondary: Color::rgb(120, 120, 120),
            brand_accent: Color::rgb(0, 245, 212),
            suggestion_bar_bg: Color::rgb(0, 0, 0),
            suggestion_chip_bg: Color::rgb(20, 20, 20),
            suggestion_chip_active: Color::rgb(0, 245, 212),
            suggestion_text: Color::rgb(220, 220, 220),
            suggestion_text_active: Color::rgb(0, 0, 0),
            popup_bg: Color::rgb(28, 28, 28),
            popup_border: Color::rgb(50, 50, 50),
            popup_text: Color::rgb(255, 255, 255),
            ripple_color: Color::rgba(0, 245, 212, 90),
            space_branding_color: Color::rgba(0, 245, 212, 200),
            divider_color: Color::rgb(25, 25, 25),
        }
    }

    pub fn sunset() -> Self {
        Self {
            id: ThemeId::Sunset,
            bg_color: Color::rgb(30, 22, 42), // Twilight Purple
            key_normal: Color::rgb(45, 34, 62),
            key_pressed: Color::rgb(65, 48, 90),
            key_modifier: Color::rgb(35, 26, 50),
            key_accent: Color::rgb(255, 107, 139), // Neon Coral / Rose
            key_accent_text: Color::rgb(255, 255, 255),
            key_shadow: Color::rgba(0, 0, 0, 80),
            text_primary: Color::rgb(250, 245, 255),
            text_secondary: Color::rgb(160, 145, 180),
            brand_accent: Color::rgb(255, 107, 139),
            suggestion_bar_bg: Color::rgb(23, 17, 32),
            suggestion_chip_bg: Color::rgb(40, 30, 56),
            suggestion_chip_active: Color::rgb(255, 107, 139),
            suggestion_text: Color::rgb(230, 220, 245),
            suggestion_text_active: Color::rgb(255, 255, 255),
            popup_bg: Color::rgb(54, 40, 77),
            popup_border: Color::rgb(80, 60, 110),
            popup_text: Color::rgb(255, 255, 255),
            ripple_color: Color::rgba(255, 107, 139, 80),
            space_branding_color: Color::rgba(255, 107, 139, 180),
            divider_color: Color::rgb(48, 36, 68),
        }
    }

    pub fn from_id(id: ThemeId) -> Self {
        match id {
            ThemeId::Dark => Self::dark(),
            ThemeId::Light => Self::light(),
            ThemeId::Amoled => Self::amoled(),
            ThemeId::Sunset => Self::sunset(),
        }
    }
}
