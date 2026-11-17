//! Explicit host records for Store consumer tests; these are not runtime defaults.

use wow_ui_sim::c_api::store_catalog_model::{
    StoreCatalogInput, StoreCurrencyInfo, StoreEntryInput, StoreProductGroupInput,
    StoreProductInput, StoreProductListResponse, StoreProductSharedData, StorePurchaseListResponse,
};
use wow_ui_sim::c_api::store_vas_model::{
    StoreVasBoostInput, StoreVasCharacterInput, StoreVasCompletionInput, StoreVasEligibleRaceInput,
    StoreVasGuildMasterInfoInput, StoreVasGuildMemberInput, StoreVasPriceInput,
    StoreVasPurchaseInput, StoreVasRaceEligibilityInput, StoreVasRealmInput, StoreVasState,
};
use wow_ui_sim::lua_api::WowLuaEnv;

const FEATURED_GROUP_ID: u32 = 501;
const FEATURED_ENTRY_ID: i32 = 1003;
const FEATURED_PRODUCT_ID: i32 = 2003;
const CURRENCY_ID: i32 = 61;

pub fn seed_vas_catalog(env: &WowLuaEnv) {
    let vas_service_type = enum_value(env, "Enum.VasServiceType.NameChange");
    let vas_decorator = enum_value(env, "Enum.BattlepayProductDecorator.VasService");
    let mut product = featured_product(0, vas_service_type);
    product.shared_data.product_decorator = Some(vas_decorator);
    let entry = featured_entry();
    let mut sim = env.state().borrow_mut();
    sim.store_glue.store_secure.catalog.products.push(product);
    sim.store_glue.store_secure.catalog.entries.push(entry);
    sim.store_glue
        .store_secure
        .vas
        .purchase_eligible_product_ids
        .insert(FEATURED_PRODUCT_ID);
}

pub fn publish_featured_catalog(env: &WowLuaEnv) {
    let card_type = enum_value(env, "Enum.BattlepayCardType.MediumCardWithBuyButton");
    let vas_service_type = enum_value(env, "Enum.VasServiceType.NameChange");
    let catalog = featured_catalog(card_type, vas_service_type);
    store_host_readiness(env, &catalog);
    receive_featured_responses(env, catalog);
}

fn store_host_readiness(env: &WowLuaEnv, catalog: &StoreCatalogInput) {
    let mut sim = env.state().borrow_mut();
    let store = &mut sim.store_glue.store_secure;
    store.catalog = catalog.clone();
    store.currency_id = CURRENCY_ID;
    store.currencies.insert(CURRENCY_ID, currency_info());
    store.distribution_list_ready = Some(true);
    sim.store_glue.connection_present = true;
    let callbacks = &mut sim.store_glue.callbacks;
    callbacks.store_status_changed = true;
    callbacks.store_products_updated = true;
    callbacks.store_purchase_list_updated = true;
}

fn receive_featured_responses(env: &WowLuaEnv, catalog: StoreCatalogInput) {
    wow_ui_sim::c_api::store_catalog_model::receive_availability(env, true)
        .expect("store availability response should dispatch");
    wow_ui_sim::c_api::store_catalog_model::receive_product_list(
        env,
        StoreProductListResponse {
            result_code: 0,
            currency_id: CURRENCY_ID,
            catalog,
        },
    )
    .expect("store product-list response should dispatch");
    wow_ui_sim::c_api::store_catalog_model::receive_purchase_list(
        env,
        StorePurchaseListResponse {
            result_code: 0,
            rows: Vec::new(),
        },
    )
    .expect("store purchase-list response should dispatch");
}

fn featured_catalog(card_type: u32, vas_service_type: u32) -> StoreCatalogInput {
    StoreCatalogInput {
        groups: vec![StoreProductGroupInput {
            group_id: FEATURED_GROUP_ID,
            parent_group_id: 0,
            order: 1,
            displayable: true,
            name: "Featured".into(),
            texture: None,
            display_type: 1,
            flags: 0,
            disabled_tooltip: None,
        }],
        products: vec![featured_product(card_type, vas_service_type)],
        entries: vec![featured_entry()],
    }
}

fn featured_product(card_type: u32, vas_service_type: u32) -> StoreProductInput {
    StoreProductInput {
        product_id: FEATURED_PRODUCT_ID,
        supported: true,
        session_eligible: true,
        display_eligible: true,
        dynamic_bundle: false,
        product_type: Some(7),
        shared_data: StoreProductSharedData {
            normal_dollars: 10,
            current_dollars: 10,
            item_quantity: 1,
            buyable_here: true,
            can_change_account: true,
            can_change_bnet_account: true,
            card_type,
            vas_service_type: Some(vas_service_type),
            name: Some("Apprentice Rider Bundle".into()),
            description: Some("Store UI test input".into()),
            tooltip: Some("Store UI test input".into()),
            ..Default::default()
        },
    }
}

fn featured_entry() -> StoreEntryInput {
    StoreEntryInput {
        entry_id: FEATURED_ENTRY_ID,
        product_id: FEATURED_PRODUCT_ID,
        group_id: FEATURED_GROUP_ID,
        order: 1,
        banner_type: 0,
        hidden: false,
        nydus_link: Some("fixture-link".into()),
    }
}

fn currency_info() -> StoreCurrencyInfo {
    StoreCurrencyInfo {
        region_id: 1,
        format_short: "short-%s".into(),
        format_long: "long-%s".into(),
        license_accept_text: "test terms".into(),
        require_license_accept: false,
        browse_has_star: false,
        hide_browse_notice: true,
        hide_confirmation_browse_notice: false,
    }
}

fn enum_value(env: &WowLuaEnv, path: &str) -> u32 {
    let value: i64 = env
        .eval(&format!("return {path}"))
        .unwrap_or_else(|error| panic!("{path} should be available: {error}"));
    u32::try_from(value).unwrap_or_else(|_| panic!("{path} should be a non-negative enum value"))
}

pub fn seed_store_vas_state(env: &WowLuaEnv) {
    let service_type = u8::try_from(enum_value(env, "Enum.VasServiceType.NameChange"))
        .expect("NameChange service type should fit the host row");
    let mut sim = env.state().borrow_mut();
    sim.store_glue.connection_present = true;
    let vas = &mut sim.store_glue.store_secure.vas;
    seed_store_vas_lookup_inputs(vas, service_type);
    seed_store_vas_purchase_inputs(vas);
}

fn seed_store_vas_lookup_inputs(vas: &mut StoreVasState, service_type: u8) {
    vas.realms = build_store_api_realms();
    vas.vas_realms = vas.realms.clone();
    let characters = vec![
        build_store_api_character(1, "Simhero", "Player-101-1-11", 70),
        build_store_api_character(2, "Simalt", "Player-101-1-12", 60),
    ];
    vas.characters = characters.clone();
    vas.character_objects = characters;
    vas.guild_character_guids.insert("Player-101-1-11".into());
    vas.local_wow_account_guid = Some("WowAccount-1001-1".into());
    vas.wow_account_guids
        .insert(b"WoW2".to_vec(), "WowAccount-2002-2".into());
    vas.race_eligibility = vec![build_store_api_races(service_type)];
    vas.guild_master_info = vec![build_store_api_guild_info()];
}

fn seed_store_vas_purchase_inputs(vas: &mut StoreVasState) {
    seed_store_vas_confirmation_inputs(vas);
    seed_store_vas_completion_inputs(vas);
    vas.failure_marker = 1;
    vas.received_failure_code = 1;
    vas.pre_generated_external_transaction_id = Some(b"external-1".to_vec());
}

fn seed_store_vas_confirmation_inputs(vas: &mut StoreVasState) {
    vas.purchase_records = vec![StoreVasPurchaseInput {
        product_id: FEATURED_PRODUCT_ID,
        status: 9,
        secondary_status: 0,
        wallet_name: "Balance".into(),
        original_amount: 0,
        current_amount: 0,
    }];
    vas.selected_purchase_product_id = Some(FEATURED_PRODUCT_ID);
    vas.product_prices.insert(
        FEATURED_PRODUCT_ID,
        StoreVasPriceInput {
            original_amount: 100000,
            current_amount: 50100,
        },
    );
}

fn seed_store_vas_completion_inputs(vas: &mut StoreVasState) {
    vas.boost = Some(StoreVasBoostInput {
        product_id: FEATURED_PRODUCT_ID,
        character_guid: "Player-101-1-11".into(),
        realm_address: 101,
    });
    vas.completion = Some(StoreVasCompletionInput {
        product_id: FEATURED_PRODUCT_ID,
        guid: "Player-101-1-11".into(),
        first_auxiliary: 0,
        second_auxiliary: 0,
    });
}

fn build_store_api_realms() -> Vec<StoreVasRealmInput> {
    vec![
        StoreVasRealmInput {
            virtual_realm_address: 101,
            realm_name: "Azeroth".into(),
            character_count: 2,
            pvp: Some(false),
            rp: Some(false),
        },
        StoreVasRealmInput {
            virtual_realm_address: 102,
            realm_name: "Kalimdor".into(),
            character_count: 1,
            pvp: Some(true),
            rp: Some(false),
        },
    ]
}

fn build_store_api_character(
    order: u64,
    name: &str,
    guid: &str,
    level: u32,
) -> StoreVasCharacterInput {
    StoreVasCharacterInput {
        realm_address: 101,
        realm_name: Some("Azeroth".into()),
        guid: Some(guid.into()),
        guid_order: Some(order),
        name: Some(name.into()),
        level,
        ..Default::default()
    }
}

fn build_store_api_races(service_type: u8) -> StoreVasRaceEligibilityInput {
    StoreVasRaceEligibilityInput {
        guid: "Player-101-1-11".into(),
        service_type,
        results: Some(vec![
            StoreVasEligibleRaceInput {
                race_name: "Human".into(),
                is_allied_race: false,
                is_heritage_armor_unlocked: false,
            },
            StoreVasEligibleRaceInput {
                race_name: "Void Elf".into(),
                is_allied_race: true,
                is_heritage_armor_unlocked: true,
            },
        ]),
    }
}

fn build_store_api_guild_info() -> StoreVasGuildMasterInfoInput {
    StoreVasGuildMasterInfoInput {
        guid: "Player-101-1-11".into(),
        guild_name: "Simulator Guild".into(),
        members: vec![StoreVasGuildMemberInput {
            guid: Some("Player-101-1-12".into()),
            member_name: "Simalt".into(),
        }],
    }
}
