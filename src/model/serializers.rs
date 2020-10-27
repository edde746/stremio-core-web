use crate::env::WebEnv;
use crate::model::deep_links::{
    AddonsDeepLinks, DiscoverDeepLinks, LibraryDeepLinks, LibraryItemDeepLinks, MetaItemDeepLinks,
    StreamDeepLinks, VideoDeepLinks,
};
use either::Either;
use itertools::Itertools;
use serde::Serialize;
use std::iter;
use stremio_core::constants::{CATALOG_PAGE_SIZE, META_RESOURCE_NAME, SKIP_EXTRA_NAME};
use stremio_core::models::catalog_with_filters::{
    CatalogWithFilters, Selected as CatalogWithFiltersSelected,
};
use stremio_core::models::catalogs_with_extra::{
    CatalogsWithExtra, Selected as CatalogsWithExtraSelected,
};
use stremio_core::models::common::{Loadable, ResourceError};
use stremio_core::models::continue_watching_preview::ContinueWatchingPreview;
use stremio_core::models::ctx::Ctx;
use stremio_core::models::installed_addons_with_filters::{
    InstalledAddonsRequest, InstalledAddonsWithFilters,
    Selected as InstalledAddonsWithFiltersSelected,
};
use stremio_core::models::library_with_filters::{
    LibraryWithFilters, Selected as LibraryWithFiltersSelected, Sort,
};
use stremio_core::models::meta_details::{MetaDetails, Selected as MetaDetailsSelected};
use stremio_core::runtime::Env;
use stremio_core::types::addon::{DescriptorPreview, ResourceRequest};
use stremio_core::types::library::LibraryItem;
use stremio_core::types::resource::{MetaItem, MetaItemPreview, Stream, Video};
use url::Url;
use wasm_bindgen::JsValue;

pub fn serialize_catalogs_with_extra(
    catalogs_with_extra: &CatalogsWithExtra,
    ctx: &Ctx<WebEnv>,
) -> JsValue {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _Stream<'a> {
        #[serde(flatten)]
        stream: &'a Stream,
        deep_links: StreamDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _MetaItemPreview<'a> {
        #[serde(flatten)]
        meta_item: &'a MetaItemPreview,
        trailer_streams: Vec<_Stream<'a>>,
        deep_links: MetaItemDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _ResourceLoadable<'a> {
        request: &'a ResourceRequest,
        content: Loadable<Vec<_MetaItemPreview<'a>>, &'a ResourceError>,
        addon_name: Option<&'a String>,
        deep_links: DiscoverDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _CatalogsWithExtra<'a> {
        selected: &'a Option<CatalogsWithExtraSelected>,
        catalogs: Vec<_ResourceLoadable<'a>>,
    }
    JsValue::from_serde(&_CatalogsWithExtra {
        selected: &catalogs_with_extra.selected,
        catalogs: catalogs_with_extra
            .catalogs
            .iter()
            .map(|catalog| _ResourceLoadable {
                request: &catalog.request,
                content: match &catalog.content {
                    Loadable::Ready(meta_items) => Loadable::Ready(
                        meta_items
                            .iter()
                            .map(|meta_item| _MetaItemPreview {
                                meta_item,
                                trailer_streams: meta_item
                                    .trailer_streams
                                    .iter()
                                    .map(|stream| _Stream {
                                        stream,
                                        deep_links: StreamDeepLinks::from(stream),
                                    })
                                    .collect::<Vec<_>>(),
                                deep_links: MetaItemDeepLinks::from(meta_item),
                            })
                            .collect::<Vec<_>>(),
                    ),
                    Loadable::Loading => Loadable::Loading,
                    Loadable::Err(error) => Loadable::Err(&error),
                },
                addon_name: ctx
                    .profile
                    .addons
                    .iter()
                    .find(|addon| addon.transport_url == catalog.request.base)
                    .map(|addon| &addon.manifest.name),
                deep_links: DiscoverDeepLinks::from(&catalog.request),
            })
            .collect::<Vec<_>>(),
    })
    .unwrap()
}

pub fn serialize_library<F>(library: &LibraryWithFilters<F>, root: String) -> JsValue {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _LibraryItem<'a> {
        #[serde(flatten)]
        library_item: &'a LibraryItem,
        deep_links: LibraryItemDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _SelectableType<'a> {
        r#type: &'a Option<String>,
        selected: &'a bool,
        deep_links: LibraryDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _SelectableSort<'a> {
        sort: &'a Sort,
        selected: &'a bool,
        deep_links: LibraryDeepLinks,
    }
    #[derive(Serialize)]
    struct _Selectable<'a> {
        types: Vec<_SelectableType<'a>>,
        sorts: Vec<_SelectableSort<'a>>,
    }
    #[derive(Serialize)]
    struct _LibraryWithFilters<'a> {
        selected: &'a Option<LibraryWithFiltersSelected>,
        selectable: _Selectable<'a>,
        catalog: Vec<_LibraryItem<'a>>,
    }
    JsValue::from_serde(&_LibraryWithFilters {
        selected: &library.selected,
        selectable: _Selectable {
            types: library
                .selectable
                .types
                .iter()
                .map(|selectable_type| _SelectableType {
                    r#type: &selectable_type.r#type,
                    selected: &selectable_type.selected,
                    deep_links: LibraryDeepLinks::from((&root, &selectable_type.request)),
                })
                .collect(),
            sorts: library
                .selectable
                .sorts
                .iter()
                .map(|selectable_sort| _SelectableSort {
                    sort: &selectable_sort.sort,
                    selected: &selectable_sort.selected,
                    deep_links: LibraryDeepLinks::from((&root, &selectable_sort.request)),
                })
                .collect(),
        },
        catalog: library
            .catalog
            .iter()
            .map(|library_item| _LibraryItem {
                library_item,
                deep_links: LibraryItemDeepLinks::from(library_item),
            })
            .collect(),
    })
    .unwrap()
}

pub fn serialize_continue_watching_preview(
    continue_watching_preview: &ContinueWatchingPreview,
) -> JsValue {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _LibraryItem<'a> {
        #[serde(flatten)]
        library_item: &'a LibraryItem,
        deep_links: LibraryItemDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _ContinueWatchingPreview<'a> {
        library_items: Vec<_LibraryItem<'a>>,
        deep_links: LibraryDeepLinks,
    }
    JsValue::from_serde(&_ContinueWatchingPreview {
        library_items: continue_watching_preview
            .library_items
            .iter()
            .map(|library_item| _LibraryItem {
                library_item,
                deep_links: LibraryItemDeepLinks::from(library_item),
            })
            .collect::<Vec<_>>(),
        deep_links: LibraryDeepLinks::from(&"continuewatching".to_owned()),
    })
    .unwrap()
}

pub fn serialize_discover(
    discover: &CatalogWithFilters<MetaItemPreview>,
    ctx: &Ctx<WebEnv>,
) -> JsValue {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _SelectableExtraOption<'a> {
        value: &'a Option<String>,
        selected: &'a bool,
        deep_links: DiscoverDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _SelectableExtra<'a> {
        name: &'a String,
        is_required: &'a bool,
        options: Vec<_SelectableExtraOption<'a>>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _SelectableCatalog<'a> {
        catalog: &'a String,
        addon_name: &'a String,
        selected: &'a bool,
        request: &'a ResourceRequest,
        deep_links: DiscoverDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _SelectableType<'a> {
        r#type: &'a String,
        selected: &'a bool,
        request: &'a ResourceRequest,
        deep_links: DiscoverDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _SelectablePage {
        deep_links: DiscoverDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _Selectable<'a> {
        types: Vec<_SelectableType<'a>>,
        catalogs: Vec<_SelectableCatalog<'a>>,
        extra: Vec<_SelectableExtra<'a>>,
        prev_page: Option<_SelectablePage>,
        next_page: Option<_SelectablePage>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _Stream<'a> {
        #[serde(flatten)]
        stream: &'a Stream,
        deep_links: StreamDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _MetaItemPreview<'a> {
        #[serde(flatten)]
        meta_item: &'a MetaItemPreview,
        trailer_streams: Vec<_Stream<'a>>,
        in_library: bool,
        deep_links: MetaItemDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _ResourceLoadable<'a> {
        content: Loadable<Vec<_MetaItemPreview<'a>>, &'a ResourceError>,
        addon_name: Option<&'a String>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _CatalogWithFilters<'a> {
        selected: &'a Option<CatalogWithFiltersSelected>,
        selectable: _Selectable<'a>,
        catalog: Option<_ResourceLoadable<'a>>,
        default_request: Option<&'a ResourceRequest>,
        page: u32,
    }
    JsValue::from_serde(&_CatalogWithFilters {
        selected: &discover.selected,
        selectable: _Selectable {
            types: discover
                .selectable
                .types
                .iter()
                .map(|selectable_type| _SelectableType {
                    r#type: &selectable_type.r#type,
                    selected: &selectable_type.selected,
                    request: &selectable_type.request,
                    deep_links: DiscoverDeepLinks::from(&selectable_type.request),
                })
                .collect(),
            catalogs: discover
                .selectable
                .catalogs
                .iter()
                .map(|selectable_catalog| _SelectableCatalog {
                    catalog: &selectable_catalog.catalog,
                    addon_name: &selectable_catalog.addon_name,
                    selected: &selectable_catalog.selected,
                    request: &selectable_catalog.request,
                    deep_links: DiscoverDeepLinks::from(&selectable_catalog.request),
                })
                .collect(),
            extra: discover
                .selectable
                .extra
                .iter()
                .map(|selectable_extra| _SelectableExtra {
                    name: &selectable_extra.name,
                    is_required: &selectable_extra.is_required,
                    options: selectable_extra
                        .options
                        .iter()
                        .map(|option| _SelectableExtraOption {
                            value: &option.value,
                            selected: &option.selected,
                            deep_links: DiscoverDeepLinks::from(&option.request),
                        })
                        .collect(),
                })
                .collect(),
            prev_page: discover
                .selectable
                .prev_page
                .as_ref()
                .map(|prev_page| _SelectablePage {
                    deep_links: DiscoverDeepLinks::from(&prev_page.request),
                }),
            next_page: discover
                .selectable
                .next_page
                .as_ref()
                .map(|next_page| _SelectablePage {
                    deep_links: DiscoverDeepLinks::from(&next_page.request),
                }),
        },
        catalog: discover.catalog.as_ref().map(|catalog| _ResourceLoadable {
            content: match &catalog.content {
                Loadable::Ready(meta_items) => Loadable::Ready(
                    meta_items
                        .iter()
                        .map(|meta_item| _MetaItemPreview {
                            meta_item,
                            trailer_streams: meta_item
                                .trailer_streams
                                .iter()
                                .map(|stream| _Stream {
                                    stream,
                                    deep_links: StreamDeepLinks::from(stream),
                                })
                                .collect::<Vec<_>>(),
                            in_library: ctx
                                .library
                                .items
                                .get(&meta_item.id)
                                .map(|library_item| !library_item.removed)
                                .unwrap_or_default(),
                            deep_links: MetaItemDeepLinks::from(meta_item),
                        })
                        .collect::<Vec<_>>(),
                ),
                Loadable::Loading => Loadable::Loading,
                Loadable::Err(error) => Loadable::Err(&error),
            },
            addon_name: ctx
                .profile
                .addons
                .iter()
                .find(|addon| addon.transport_url == catalog.request.base)
                .map(|addon| &addon.manifest.name),
        }),
        default_request: discover
            .selectable
            .types
            .first()
            .map(|first_type| &first_type.request),
        page: discover
            .selected
            .as_ref()
            .and_then(|selected| {
                selected
                    .request
                    .path
                    .get_extra_first_value(SKIP_EXTRA_NAME)
                    .and_then(|value| value.parse::<u32>().ok())
                    .map(|skip| 1 + skip / CATALOG_PAGE_SIZE as u32)
            })
            .unwrap_or(1),
    })
    .unwrap()
}

pub fn serialize_remote_addons(
    remote_addons: &CatalogWithFilters<DescriptorPreview>,
    ctx: &Ctx<WebEnv>,
) -> JsValue {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _SelectableCatalog<'a> {
        catalog: &'a String,
        addon_name: &'a String,
        selected: &'a bool,
        request: &'a ResourceRequest,
        deep_links: AddonsDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _SelectableType<'a> {
        r#type: &'a String,
        selected: &'a bool,
        request: &'a ResourceRequest,
        deep_links: AddonsDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _Selectable<'a> {
        catalogs: Vec<_SelectableCatalog<'a>>,
        types: Vec<_SelectableType<'a>>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _DescriptorPreview<'a> {
        #[serde(flatten)]
        addon: &'a DescriptorPreview,
        installed: bool,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _ResourceLoadable<'a> {
        content: Loadable<Vec<_DescriptorPreview<'a>>, &'a ResourceError>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _CatalogWithFilters<'a> {
        selected: &'a Option<CatalogWithFiltersSelected>,
        selectable: _Selectable<'a>,
        catalog: Option<_ResourceLoadable<'a>>,
    }
    JsValue::from_serde(&_CatalogWithFilters {
        selected: &remote_addons.selected,
        selectable: _Selectable {
            catalogs: remote_addons
                .selectable
                .catalogs
                .iter()
                .map(|selectable_catalog| _SelectableCatalog {
                    catalog: &selectable_catalog.catalog,
                    addon_name: &selectable_catalog.addon_name,
                    selected: &selectable_catalog.selected,
                    request: &selectable_catalog.request,
                    deep_links: AddonsDeepLinks::from(&selectable_catalog.request),
                })
                .collect(),
            types: remote_addons
                .selectable
                .types
                .iter()
                .map(|selectable_type| _SelectableType {
                    r#type: &selectable_type.r#type,
                    selected: &selectable_type.selected,
                    request: &selectable_type.request,
                    deep_links: AddonsDeepLinks::from(&selectable_type.request),
                })
                .collect(),
        },
        catalog: remote_addons
            .catalog
            .as_ref()
            .map(|catalog| _ResourceLoadable {
                content: match &catalog.content {
                    Loadable::Ready(addons) => Loadable::Ready(
                        addons
                            .iter()
                            .map(|addon| _DescriptorPreview {
                                addon,
                                installed: ctx
                                    .profile
                                    .addons
                                    .iter()
                                    .map(|addon| &addon.transport_url)
                                    .any(|transport_url| *transport_url == addon.transport_url),
                            })
                            .collect::<Vec<_>>(),
                    ),
                    Loadable::Loading => Loadable::Loading,
                    Loadable::Err(error) => Loadable::Err(&error),
                },
            }),
    })
    .unwrap()
}

pub fn serialize_installed_addons(installed_addons: &InstalledAddonsWithFilters) -> JsValue {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _DescriptorPreview<'a> {
        #[serde(flatten)]
        addon: &'a DescriptorPreview,
        installed: bool,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _SelectableType<'a> {
        r#type: &'a Option<String>,
        selected: &'a bool,
        deep_links: AddonsDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _SelectableCatalog {
        catalog: String,
        selected: bool,
        deep_links: AddonsDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _Selectable<'a> {
        types: Vec<_SelectableType<'a>>,
        catalogs: Vec<_SelectableCatalog>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _InstalledAddonsWithFilters<'a> {
        selected: &'a Option<InstalledAddonsWithFiltersSelected>,
        selectable: _Selectable<'a>,
        catalog: Vec<_DescriptorPreview<'a>>,
    }
    JsValue::from_serde(&_InstalledAddonsWithFilters {
        selected: &installed_addons.selected,
        selectable: _Selectable {
            types: installed_addons
                .selectable
                .types
                .iter()
                .map(|selectable_type| _SelectableType {
                    r#type: &selectable_type.r#type,
                    selected: &selectable_type.selected,
                    deep_links: AddonsDeepLinks::from(&selectable_type.request),
                })
                .collect(),
            catalogs: vec![_SelectableCatalog {
                catalog: "Installed".to_owned(),
                selected: installed_addons.selected.is_some(),
                deep_links: AddonsDeepLinks::from(&InstalledAddonsRequest { r#type: None }),
            }],
        },
        catalog: installed_addons
            .catalog
            .iter()
            .map(|addon| _DescriptorPreview {
                addon,
                installed: true,
            })
            .collect(),
    })
    .unwrap()
}

pub fn serialize_meta_details(meta_details: &MetaDetails, ctx: &Ctx<WebEnv>) -> JsValue {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _ManifestPreview<'a> {
        name: &'a String,
        logo: &'a Option<String>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _DescriptorPreview<'a> {
        manifest: _ManifestPreview<'a>,
        transport_url: &'a Url,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _Video<'a> {
        #[serde(flatten)]
        video: &'a Video,
        trailer_streams: Vec<_Stream<'a>>,
        upcomming: bool,
        watched: bool,
        progress: Option<u32>,
        scheduled: bool,
        deep_links: VideoDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _MetaItem<'a> {
        #[serde(flatten)]
        meta_item: &'a MetaItem,
        videos: Vec<_Video<'a>>,
        trailer_streams: Vec<_Stream<'a>>,
        in_library: bool,
        deep_links: MetaItemDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _Stream<'a> {
        #[serde(flatten)]
        stream: &'a Stream,
        deep_links: StreamDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _MetaExtension<'a> {
        url: &'a Url,
        name: &'a String,
        addon: _DescriptorPreview<'a>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _ResourceLoadable<'a, T> {
        content: Loadable<T, &'a ResourceError>,
        addon_name: Option<&'a String>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _MetaDetails<'a> {
        selected: &'a Option<MetaDetailsSelected>,
        meta_catalog: Option<_ResourceLoadable<'a, _MetaItem<'a>>>,
        streams_catalogs: Vec<_ResourceLoadable<'a, Vec<_Stream<'a>>>>,
        meta_extensions: Vec<_MetaExtension<'a>>,
    }
    let meta_catalog = meta_details
        .meta_catalogs
        .iter()
        .find(|catalog| catalog.content.is_ready())
        .or_else(|| {
            if meta_details
                .meta_catalogs
                .iter()
                .all(|catalog| catalog.content.is_err())
            {
                meta_details.meta_catalogs.first()
            } else {
                meta_details
                    .meta_catalogs
                    .iter()
                    .find(|catalog| catalog.content.is_loading())
            }
        });
    JsValue::from_serde(&_MetaDetails {
        selected: &meta_details.selected,
        meta_catalog: meta_catalog.map(|catalog| _ResourceLoadable {
            content: match &catalog.content {
                Loadable::Ready(meta_item) => Loadable::Ready(_MetaItem {
                    meta_item,
                    videos: meta_item
                        .videos
                        .iter()
                        .map(|video| _Video {
                            video,
                            trailer_streams: video
                                .trailer_streams
                                .iter()
                                .map(|stream| _Stream {
                                    stream,
                                    deep_links: StreamDeepLinks::from(stream),
                                })
                                .collect::<Vec<_>>(),
                            upcomming: meta_item.behavior_hints.has_scheduled_videos
                                && meta_item
                                    .released
                                    .map(|released| released > WebEnv::now())
                                    .unwrap_or(true),
                            watched: false, // TODO use library
                            progress: None, // TODO use library,
                            scheduled: meta_item.behavior_hints.has_scheduled_videos,
                            deep_links: VideoDeepLinks::from((video, &catalog.request)),
                        })
                        .collect::<Vec<_>>(),
                    trailer_streams: meta_item
                        .trailer_streams
                        .iter()
                        .map(|stream| _Stream {
                            stream,
                            deep_links: StreamDeepLinks::from(stream),
                        })
                        .collect::<Vec<_>>(),
                    in_library: ctx
                        .library
                        .items
                        .get(&meta_item.id)
                        .map(|library_item| !library_item.removed)
                        .unwrap_or_default(),
                    deep_links: MetaItemDeepLinks::from(meta_item),
                }),
                Loadable::Loading => Loadable::Loading,
                Loadable::Err(error) => Loadable::Err(&error),
            },
            addon_name: ctx
                .profile
                .addons
                .iter()
                .find(|addon| addon.transport_url == catalog.request.base)
                .map(|addon| &addon.manifest.name),
        }),
        streams_catalogs: meta_details
            .streams_catalogs
            .iter()
            .map(|catalog| _ResourceLoadable {
                content: match &catalog.content {
                    Loadable::Ready(streams) => Loadable::Ready(
                        streams
                            .iter()
                            .map(|stream| _Stream {
                                stream,
                                deep_links: meta_catalog.map_or_else(
                                    || StreamDeepLinks::from(stream),
                                    |meta_catalog| {
                                        StreamDeepLinks::from((
                                            stream,
                                            &catalog.request,
                                            &meta_catalog.request,
                                        ))
                                    },
                                ),
                            })
                            .collect::<Vec<_>>(),
                    ),
                    Loadable::Loading => Loadable::Loading,
                    Loadable::Err(error) => Loadable::Err(&error),
                },
                addon_name: ctx
                    .profile
                    .addons
                    .iter()
                    .find(|addon| addon.transport_url == catalog.request.base)
                    .map(|addon| &addon.manifest.name),
            })
            .collect::<Vec<_>>(),
        meta_extensions: meta_details
            .meta_catalogs
            .iter()
            .flat_map(|catalog| match &catalog.content {
                Loadable::Ready(meta_item) => Either::Left(
                    meta_item
                        .links
                        .iter()
                        .filter(|link| link.category == META_RESOURCE_NAME)
                        .map(move |link| (&catalog.request, link)),
                ),
                _ => Either::Right(iter::empty()),
            })
            .unique_by(|(_, link)| &link.url)
            .filter_map(|(request, link)| {
                ctx.profile
                    .addons
                    .iter()
                    .find(|addon| addon.transport_url == request.base)
                    .map(|addon| _MetaExtension {
                        url: &link.url,
                        name: &link.name,
                        addon: _DescriptorPreview {
                            transport_url: &addon.transport_url,
                            manifest: _ManifestPreview {
                                name: &addon.manifest.name,
                                logo: &addon.manifest.logo,
                            },
                        },
                    })
            })
            .collect::<Vec<_>>(),
    })
    .unwrap()
}
