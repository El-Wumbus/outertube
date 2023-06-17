use std::fmt;

const REFERER_YOUTUBE: &str = "https://www.youtube.com/";
const REFERER_YOUTUBE_MOBILE: &str = "https://m.youtube.com/";
const REFERER_YOUTUBE_MUSIC: &str = "https://music.youtube.com/";
const REFERER_YOUTUBE_KIDS: &str = "https://www.youtubekids.com/";
const REFERER_YOUTUBE_STUDIO: &str = "https://studio.youtube.com/";
const REFERER_YOUTUBE_ANALYTICS: &str = "https://analytics.youtube.com/";
// const REFERER_GOOGLE_ASSISTANT: &str = "https://assistant.google.com/";

const USER_AGENT_WEB:& str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/74.0.3729.157 Safari/537.36";
// const USER_AGEN_WEB_ALT:&str = "Mozilla/5.0 (X11; Linux x86_64; rv:102.0)
// Gecko/20100101 Firefox/102.0";
const USER_AGENT_ANDROID:& str =
    "Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/65.0.3325.181 Mobile Safari/537.36";
const USER_AGENT_IOS:& str =
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_4_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) FxiOS/98.2  Mobile/15E148 Safari/605.1.15";
const USER_AGENT_TV_HTML5: &str = "Mozilla/5.0 (PlayStation 4 5.55) AppleWebKit/601.2 (KHTML, like Gecko)";
const USER_AGENT_TV_APPLE: &str =
    "AppleCoreMedia/1.0.0.12B466 (Apple TV; U; CPU OS 8_1_3 like Mac OS X; en_us)";
const USER_AGENT_TV_ANDROID: &str =
    "Mozilla/5.0 (Linux; Android 5.1.1; AFTT Build/LVY48F; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/49.0.2623.10";
const USER_AGENT_XBOX_ONE: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; Xbox; Xbox One) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/46.0.2486.0 Safari/537.36 Edge/13.10553";
const USER_AGENT_GOOGLE_ASSISTANT: &str =
    "Mozilla/5.0 (Linux; Android 11; Pixel 2; DuplexWeb-Google/1.0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/86.0.4240.193 Mobile Safari/537.36";

pub struct Config {
    pub(crate) base_url:   &'static str,
    client_configurations: &'static [ClientContext],
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[repr(usize)]
pub enum ClientVariant {
    /// The YouTube web client found at `www.youtube.com`
    #[default]
    Web                  = 0,
    Mweb                 = 1,
    Android              = 2,
    Ios                  = 3,
    Tvhtml5              = 4,
    Tvlite               = 5,
    Tvandroid            = 6,
    Xboxoneguide         = 7,
    AndroidCreator       = 8,
    IosCreator           = 9,
    Tvapple              = 10,
    AndroidKids          = 11,
    IosKids              = 12,
    AndroidMusic         = 13,
    AndroidTv            = 14,
    IosMusic             = 15,
    MwebTier2            = 16,
    AndroidVr            = 17,
    AndroidUnplugged     = 18,
    AndroidTestsuite     = 19,
    WebMusicAnalytics    = 20,
    IosUnplugged         = 21,
    AndroidLite          = 22,
    IosEmbeddedPlayer    = 23,
    WebUnplugged         = 24,
    WebExperiments       = 25,
    Tvhtml5Cast          = 26,
    AndroidEmbeddedPlayer = 27,
    WebEmbeddedPlayer    = 28,
    Tvhtml5Audio         = 29,
    TvUnpluggedCast      = 30,
    Tvhtml5Kids          = 31,
    WebHeroes            = 32,
    WebMusic             = 33,
    WebCreator           = 34,
    TvUnpluggedAndroid   = 35,
    IosLiveCreationExtension = 36,
    Tvhtml5Unplugged     = 37,
    IosMessagesExtension = 38,
    WebRemix             = 39,
    IosUptime            = 40,
    WebUnpluggedOnboarding = 41,
    WebUnpluggedOps      = 42,
    WebUnpluggedPublic   = 43,
    Tvhtml5Vr            = 44,
    AndroidTvKids        = 45,
    Tvhtml5Simply        = 46,
    WebKids              = 47,
    MusicIntegrations    = 48,
    Tvhtml5Yongle        = 49,
    GoogleAssistant      = 50,
    Tvhtml5SimplyEmbeddedPlayer = 51,
    WebInternalAnalytics = 52,
    WebParentTools       = 53,
    GoogleMediaActions   = 54,
    WebPhoneVerification = 55,
    IosProducer          = 56,
    Tvhtml5ForKids       = 57,
}

impl ClientVariant {
    #[cfg(test)]
    pub(crate) fn try_from(value: usize) -> Result<Self, String> {
        if value > ClientVariant::Tvhtml5ForKids as usize {
            return Err(format!("{value} is not a variant of ClientVariant"));
        }

        Ok(unsafe { std::mem::transmute(value) })
    }
}

impl From<ClientVariant> for ClientContext {
    fn from(val: ClientVariant) -> Self {
        // Each variant is in order of the slice and serves as an index into it. It shouldn't fail
        // 🙏.
        CONFIG.client_configurations[val as usize]
    }
}

// Represents a client communicating with YouTube
#[derive(Debug, Clone, Copy)]
pub(crate) struct ClientContext {
    pub(super) name:       &'static str,
    pub(super) version:    &'static str,
    pub(super) id:         u8,
    pub(super) user_agent: Option<&'static str>,
    pub(super) api_key:    Option<&'static str>,
    pub(super) referer:    Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Locale {
    pub(crate) hl: String,
    pub(crate) gl: Option<String>,
}

impl From<&str> for Locale {
    fn from(value: &str) -> Self {
        if let Some((hl, gl)) = value.split_once('-') {
            Self {
                hl: hl.to_string(),
                gl: Some(gl.into()),
            }
        } else {
            Self {
                hl: value.to_string(),
                gl: None,
            }
        }
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.gl {
            Some(gl) => write!(f, "{}-{gl}", self.hl),
            None => {
                write!(f, "{}", self.hl)
            }
        }
    }
}

pub(crate) const CONFIG: Config = Config {
    // base_url:              "https://youtubei.googleapis.com/youtubei/v1/",
    base_url:              "https://www.youtube.com/youtubei/v1/",
    client_configurations: &[
        ClientContext {
            id:         1,
            name:       "WEB",
            version:    "2.20230607.06.00",
            //  "2.20230607.06.00"
            user_agent: Some(USER_AGENT_WEB),
            referer:    Some(REFERER_YOUTUBE),
            api_key:    Some("AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8"),
        },
        ClientContext {
            id:         2,
            name:       "MWEB",
            version:    "2.20211214.00.00",
            user_agent: Some(USER_AGENT_ANDROID),
            referer:    Some(REFERER_YOUTUBE_MOBILE),
            api_key:    Some("AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8"),
        },
        ClientContext {
            id:         3,
            name:       "ANDROID",
            version:    "17.13.3",
            user_agent: Some(USER_AGENT_ANDROID),
            api_key:    Some("AIzaSyA8eiZmM1FaDVjRy-df2KTyQ_vz_yYM39w"),
            referer:    None,
        },
        ClientContext {
            id:         5,
            name:       "IOS",
            version:    "17.14.2",
            user_agent: Some(USER_AGENT_IOS),
            api_key:    Some("AIzaSyB-63vPrdThhKuerbB2N_l7Kwwcxj6yUAc"),
            referer:    None,
        },
        ClientContext {
            id:         7,
            name:       "TVHTML5",
            version:    "7.20210224.00.00",
            user_agent: Some(USER_AGENT_TV_HTML5),
            api_key:    Some("AIzaSyDCU8hByM-4DrUqRUYnGn-3llEO78bcxq8"),
            referer:    None,
        },
        ClientContext {
            id:         8,
            name:       "TVLITE",
            version:    "2",
            user_agent: Some(USER_AGENT_TV_HTML5),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         10,
            name:       "TVANDROID",
            version:    "1.0",
            user_agent: Some(USER_AGENT_TV_ANDROID),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         13,
            name:       "XBOXONEGUIDE",
            version:    "1.0",
            user_agent: Some(USER_AGENT_XBOX_ONE),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         14,
            name:       "ANDROID_CREATOR",
            version:    "21.06.103",
            user_agent: Some(USER_AGENT_ANDROID),
            api_key:    Some("AIzaSyD_qjV8zaaUMehtLkrKFgVeSX_Iqbtyws8"),
            referer:    None,
        },
        ClientContext {
            id:         15,
            name:       "IOS_CREATOR",
            version:    "20.47.100",
            user_agent: Some(USER_AGENT_IOS),
            api_key:    Some("AIzaSyAPyF5GfQI-kOa6nZwO8EsNrGdEx9bioNs"),
            referer:    None,
        },
        ClientContext {
            id:         16,
            name:       "TVAPPLE",
            version:    "1.0",
            user_agent: Some(USER_AGENT_TV_APPLE),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         18,
            name:       "ANDROID_KIDS",
            version:    "7.12.3",
            user_agent: Some(USER_AGENT_ANDROID),
            api_key:    Some("AIzaSyAxxQKWYcEX8jHlflLt2Qcbb-rlolzBhhk"),
            referer:    None,
        },
        ClientContext {
            id:         19,
            name:       "IOS_KIDS",
            version:    "5.42.2",
            user_agent: Some(USER_AGENT_IOS),
            api_key:    Some("AIzaSyA6_JWXwHaVBQnoutCv1-GvV97-rJ949Bc"),
            referer:    None,
        },
        ClientContext {
            id:         21,
            name:       "ANDROID_MUSIC",
            version:    "5.01",
            user_agent: Some(USER_AGENT_ANDROID),
            api_key:    Some("AIzaSyAOghZGza2MQSZkY_zfZ370N-PUdXEo8AI"),
            referer:    None,
        },
        ClientContext {
            id:         23,
            name:       "ANDROID_TV",
            version:    "2.16.032",
            user_agent: Some(USER_AGENT_TV_ANDROID),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         26,
            name:       "IOS_MUSIC",
            version:    "4.16.1",
            user_agent: Some(USER_AGENT_IOS),
            api_key:    Some("AIzaSyBAETezhkwP0ZWA02RsqT1zu78Fpt0bC_s"),
            referer:    None,
        },
        ClientContext {
            id:         27,
            name:       "MWEB_TIER_2",
            version:    "9.20220325",
            user_agent: Some(USER_AGENT_ANDROID),
            referer:    Some(REFERER_YOUTUBE_MOBILE),
            api_key:    None,
        },
        ClientContext {
            id:         28,
            name:       "ANDROID_VR",
            version:    "1.28.63",
            user_agent: Some(USER_AGENT_ANDROID),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         29,
            name:       "ANDROID_UNPLUGGED",
            version:    "6.13",
            user_agent: Some(USER_AGENT_ANDROID),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         30,
            name:       "ANDROID_TESTSUITE",
            version:    "1.9",
            user_agent: Some(USER_AGENT_ANDROID),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         31,
            name:       "WEB_MUSIC_ANALYTICS",
            version:    "0.2",
            user_agent: Some(USER_AGENT_WEB),
            referer:    Some(REFERER_YOUTUBE_ANALYTICS),
            api_key:    None,
        },
        ClientContext {
            id:         33,
            name:       "IOS_UNPLUGGED",
            version:    "6.13",
            user_agent: Some(USER_AGENT_IOS),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         38,
            name:       "ANDROID_LITE",
            version:    "3.26.1",
            user_agent: Some(USER_AGENT_ANDROID),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         39,
            name:       "IOS_EMBEDDED_PLAYER",
            version:    "2.3",
            user_agent: Some(USER_AGENT_IOS),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         41,
            name:       "WEB_UNPLUGGED",
            version:    "1.20220403",
            user_agent: Some(USER_AGENT_WEB),
            referer:    Some(REFERER_YOUTUBE),
            api_key:    None,
        },
        ClientContext {
            id:         42,
            name:       "WEB_EXPERIMENTS",
            version:    "1",
            user_agent: Some(USER_AGENT_WEB),
            referer:    Some(REFERER_YOUTUBE),
            api_key:    None,
        },
        ClientContext {
            id:         43,
            name:       "TVHTML5_CAST",
            version:    "1.1",
            user_agent: Some(USER_AGENT_TV_HTML5),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         55,
            name:       "ANDROID_EMBEDDED_PLAYER",
            version:    "17.13.3",
            user_agent: Some(USER_AGENT_ANDROID),
            api_key:    Some("AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8"),
            referer:    None,
        },
        ClientContext {
            id:         56,
            name:       "WEB_EMBEDDED_PLAYER",
            version:    "1.20220413.01.00",
            user_agent: Some(USER_AGENT_WEB),
            referer:    Some(REFERER_YOUTUBE),
            api_key:    Some("AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8"),
        },
        ClientContext {
            id:         57,
            name:       "TVHTML5_AUDIO",
            version:    "2.0",
            user_agent: Some(USER_AGENT_TV_HTML5),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         58,
            name:       "TV_UNPLUGGED_CAST",
            version:    "0.1",
            user_agent: Some(USER_AGENT_TV_HTML5),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         59,
            name:       "TVHTML5_KIDS",
            version:    "3.20220325",
            user_agent: Some(USER_AGENT_TV_HTML5),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         60,
            name:       "WEB_HEROES",
            version:    "0.1",
            user_agent: Some(USER_AGENT_WEB),
            referer:    Some(REFERER_YOUTUBE),
            api_key:    None,
        },
        ClientContext {
            id:         61,
            name:       "WEB_MUSIC",
            version:    "1.0",
            user_agent: Some(USER_AGENT_WEB),
            referer:    Some(REFERER_YOUTUBE_MUSIC),
            api_key:    None,
        },
        ClientContext {
            id:         62,
            name:       "WEB_CREATOR",
            version:    "1.20210223.01.00",
            user_agent: Some(USER_AGENT_WEB),
            referer:    Some(REFERER_YOUTUBE_STUDIO),
            api_key:    Some("AIzaSyBUPetSUmoZL-OhlxA7wSac5XinrygCqMo"),
        },
        ClientContext {
            id:         63,
            name:       "TV_UNPLUGGED_ANDROID",
            version:    "1.22.062.06.90",
            user_agent: Some(USER_AGENT_TV_ANDROID),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         64,
            name:       "IOS_LIVE_CREATION_EXTENSION",
            version:    "17.13.3",
            user_agent: Some(USER_AGENT_IOS),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         65,
            name:       "TVHTML5_UNPLUGGED",
            version:    "6.13",
            user_agent: Some(USER_AGENT_TV_HTML5),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         66,
            name:       "IOS_MESSAGES_EXTENSION",
            version:    "16.20",
            user_agent: Some(USER_AGENT_IOS),
            api_key:    Some("AIzaSyDCU8hByM-4DrUqRUYnGn-3llEO78bcxq8"),
            referer:    None,
        },
        ClientContext {
            id:         67,
            name:       "WEB_REMIX",
            version:    "1.20220607.03.01",
            user_agent: Some(USER_AGENT_WEB),
            referer:    Some(REFERER_YOUTUBE_MUSIC),
            api_key:    Some("AIzaSyC9XL3ZjWddXya6X74dJoCTL-WEYFDNX30"),
        },
        ClientContext {
            id:         68,
            name:       "IOS_UPTIME",
            version:    "1.0",
            user_agent: Some(USER_AGENT_IOS),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         69,
            name:       "WEB_UNPLUGGED_ONBOARDING",
            version:    "0.1",
            user_agent: Some(USER_AGENT_WEB),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         70,
            name:       "WEB_UNPLUGGED_OPS",
            version:    "0.1",
            user_agent: Some(USER_AGENT_WEB),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         71,
            name:       "WEB_UNPLUGGED_PUBLIC",
            version:    "0.1",
            user_agent: Some(USER_AGENT_WEB),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         72,
            name:       "TVHTML5_VR",
            version:    "0.1",
            user_agent: Some(USER_AGENT_TV_HTML5),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         74,
            name:       "ANDROID_TV_KIDS",
            version:    "1.16.80",
            user_agent: Some(USER_AGENT_TV_ANDROID),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         75,
            name:       "TVHTML5_SIMPLY",
            version:    "1.0",
            user_agent: Some(USER_AGENT_TV_HTML5),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         76,
            name:       "WEB_KIDS",
            version:    "2.20220414.00.00",
            referer:    Some(REFERER_YOUTUBE_KIDS),
            user_agent: Some(USER_AGENT_WEB),
            api_key:    Some("AIzaSyBbZV_fZ3an51sF-mvs5w37OqqbsTOzwtU"),
        },
        ClientContext {
            id:         77,
            name:       "MUSIC_INTEGRATIONS",
            version:    "0.1",
            user_agent: None,
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         80,
            name:       "TVHTML5_YONGLE",
            version:    "0.1",
            user_agent: Some(USER_AGENT_TV_HTML5),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         84,
            name:       "GOOGLE_ASSISTANT",
            version:    "0.1",
            user_agent: Some(USER_AGENT_GOOGLE_ASSISTANT),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         85,
            name:       "TVHTML5_SIMPLY_EMBEDDED_PLAYER",
            version:    "2.0",
            user_agent: Some(USER_AGENT_TV_HTML5),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         87,
            name:       "WEB_INTERNAL_ANALYTICS",
            version:    "0.1",
            user_agent: Some(USER_AGENT_WEB),
            referer:    Some(REFERER_YOUTUBE_ANALYTICS),
            api_key:    None,
        },
        ClientContext {
            id:         88,
            name:       "WEB_PARENT_TOOLS",
            version:    "1.20220403",
            user_agent: Some(USER_AGENT_WEB),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         89,
            name:       "GOOGLE_MEDIA_ACTIONS",
            version:    "0.1",
            user_agent: Some(USER_AGENT_GOOGLE_ASSISTANT),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         90,
            name:       "WEB_PHONE_VERIFICATION",
            version:    "1.0.0",
            user_agent: Some(USER_AGENT_WEB),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         92,
            name:       "IOS_PRODUCER",
            version:    "0.1",
            user_agent: Some(USER_AGENT_IOS),
            api_key:    None,
            referer:    None,
        },
        ClientContext {
            id:         93,
            name:       "TVHTML5_FOR_KIDS",
            version:    "7.20220325",
            user_agent: Some(USER_AGENT_TV_HTML5),
            api_key:    None,
            referer:    None,
        },
    ],
};

// #[cfg(test)]
// mod tests {
//     use convert_case::{Case, Casing};

//     use super::*;
//     #[test]
//     fn not_real() {
//         // for x in CONFIG.client_configurations {
//         //     println!(
//         //         "ClientVariant::{} => \"{}\",",
//         //         x.name.from_case(Case::UpperSnake).to_case(Case::UpperCamel),
//         //         x.name
//         //     );
//         // }
//         for (i, x) in CONFIG.client_configurations.iter().enumerate() {
//             println!(
//                 "{} = {i},",
//                 x.name.from_case(Case::UpperSnake).to_case(Case::UpperCamel)
//             );
//         }
//         panic!();
//     }
// }
