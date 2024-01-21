use chrono::Utc;
use ::entity::entities::user_entity::user_model;
use sea_orm::*;
use ::entity::entities::user_entity::user_model::Language;
use uuid::uuid;

#[cfg(feature = "mock")]
pub fn prepare_mock_db() -> DatabaseConnection {

    MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([
            [user_model::Model {
                id: uuid!("4bcd5956-9e55-4c98-8ae6-ce03392083c4"),
                av: None,
                f: String::from("Michel"),
                l: String::from("Boittout"),
                e: String::from("michel.boittout@gmail.com"),
                ph: String::from("0630303030"),
                t: true,
                pv: true,
                two_fa: true,
                lg: Language::Fr,
                created_at: Utc::now(),
            }],
            [user_model::Model {
                id: uuid!("990e30cc-3527-45be-83b9-7fb8616097d7"),
                av: None,
                f: String::from("Fabrice"),
                l: String::from("Camisole"),
                e: String::from("fabrice.camisole@gmail.com"),
                ph: String::from("0620202020"),
                t: true,
                pv: true,
                two_fa: true,
                lg: Language::Fr,
                created_at: Utc::now(),
            }],
            [user_model::Model {
                id: uuid!("61a33650-0fbb-4636-a48e-d1b9bb9eed19"),
                av: None,
                f: String::from("Caroline"),
                l: String::from("Aphin"),
                e: String::from("caroline.aphin@gmail.com"),
                ph: String::from("0610101010"),
                t: true,
                pv: true,
                two_fa: true,
                lg: Language::Fr,
                created_at: Utc::now(),
            }],
            [user_model::Model {
                id: uuid!("4bcd5956-9e55-4c98-8ae6-ce03392083c4"),
                av: None,
                f: String::from("Michel"),
                l: String::from("Boittout"),
                e: String::from("michel.boittout@gmail.com"),
                ph: String::from("0630303030"),
                t: true,
                pv: true,
                two_fa: true,
                lg: Language::Fr,
                created_at: Utc::now(),
            }],
            [user_model::Model {
                id: uuid!("4bcd5956-9e55-4c98-8ae6-ce03392083c4"),
                av: None,
                f: String::from("Michel"),
                l: String::from("Boittout"),
                e: String::from("michel.boittout@gmail.com"),
                ph: String::from("0630303030"),
                t: true,
                pv: true,
                two_fa: true,
                lg: Language::Fr,
                created_at: Utc::now(),
            }],
            [user_model::Model {
                id: uuid!("990e30cc-3527-45be-83b9-7fb8616097d7"),
                av: None,
                f: String::from("Fabrice"),
                l: String::from("Camisole"),
                e: String::from("fabrice.camisole@gmail.com"),
                ph: String::from("0620202020"),
                t: true,
                pv: true,
                two_fa: true,
                lg: Language::Fr,
                created_at: Utc::now(),
            }],
        ])
        /* .append_exec_results([
            MockExecResult {
                last_insert_id: 6,
                rows_affected: 1,
            },
            MockExecResult {
                last_insert_id: 6,
                rows_affected: 5,
            },
        ]) */
        .into_connection()
}