use crate::model::deep_links_ext::DeepLinksExt;
use serde::Serialize;
use stremio_core::deep_links::MetaItemDeepLinks;
use stremio_core::models::common::Loadable;
use stremio_core::models::streaming_server::{PlaybackDevice, Selected, Settings, StreamingServer};
use stremio_core::runtime::EnvError;
use stremio_core::types::addon::ResourcePath;
use stremio_core::types::streaming_server::Statistics;
use url::Url;
use wasm_bindgen::JsValue;

mod model {
    use super::*;
    type TorrentLoadable<'a> = Loadable<(&'a ResourcePath, MetaItemDeepLinks), &'a EnvError>;
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct StreamingServer<'a> {
        pub selected: &'a Selected,
        pub settings: &'a Loadable<Settings, EnvError>,
        pub base_url: &'a Loadable<Url, EnvError>,
        pub playback_devices: &'a Loadable<Vec<PlaybackDevice>, EnvError>,
        pub torrent: Option<(&'a String, TorrentLoadable<'a>)>,
        pub statistics: Option<&'a Loadable<Statistics, EnvError>>,
    }
}

pub fn serialize_streaming_server(streaming_server: &StreamingServer) -> JsValue {
    JsValue::from_serde(&model::StreamingServer {
        selected: &streaming_server.selected,
        settings: &streaming_server.settings,
        base_url: &streaming_server.base_url,
        playback_devices: &streaming_server.playback_devices,
        torrent: streaming_server
            .torrent
            .as_ref()
            .map(|(info_hash, loadable)| {
                let loadable = match loadable {
                    Loadable::Ready(resource_path) => Loadable::Ready((
                        resource_path,
                        MetaItemDeepLinks::from(resource_path).into_web_deep_links(),
                    )),
                    Loadable::Loading => Loadable::Loading,
                    Loadable::Err(error) => Loadable::Err(error),
                };
                (info_hash, loadable)
            }),
        statistics: streaming_server.statistics.as_ref(),
    })
    .unwrap()
}
