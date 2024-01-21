mod prepare;

use chrono::{DateTime, FixedOffset, Utc};
use entity::user_entity::user_model::{self, Language};
use prepare::prepare_mock_db;
use service::{mutation::user_mutations::UserMutation, query::user_queries::UserQuery};
use uuid::uuid;

#[tokio::test]
async fn main() {
    let db = &prepare_mock_db();

    {
        let michel_uuid = uuid!("4bcd5956-9e55-4c98-8ae6-ce03392083c4");
        let user = UserQuery::find_user_by_id(db, michel_uuid.to_owned())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(user.id, michel_uuid.to_owned());
    }

    {
        let fabrice_uuid = uuid!("990e30cc-3527-45be-83b9-7fb8616097d7");
        let user = UserQuery::find_user_by_id(db, fabrice_uuid.to_owned())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(user.id, fabrice_uuid.to_owned());
    }

    /* {
        let user = Mutation::create_user(
            db,
            user_model::Model {
                id: uuid!("f2386571-ae4c-4d75-92ec-38e049d27a10"),
                av: None,
                f: String::from("Fredo"),
                l: String::from("Lafrite"),
                e: String::from("fredo.lafrite@gmail.com"),
                ph: String::from("0611111111"),
                t: true,
                pv: true,
                two_fa: true,
                lg: Language::Fr,
                created_at: Utc::now(),
            },
        )
        .await
        .unwrap();
        let input = "2023-08-31T16:11:21.053123200Z";

        let parsed_datetime: DateTime<FixedOffset> =
            DateTime::parse_from_rfc3339(input).expect("Invalid date format");

        let datetime_utc: DateTime<Utc> = parsed_datetime.with_timezone(&Utc);

        assert_eq!(
            user,
            user_model::ActiveModel {
                id: sea_orm::ActiveValue::Unchanged(uuid!("61a33650-0fbb-4636-a48e-d1b9bb9eed19")),
                av: sea_orm::ActiveValue::Unchanged(None),
                f: sea_orm::ActiveValue::Unchanged("Caroline".to_owned()),
                l: sea_orm::ActiveValue::Unchanged("Aphin".to_owned()),
                e: sea_orm::ActiveValue::Unchanged("caroline.aphin@gmail.com".to_owned()),
                ph: sea_orm::ActiveValue::Unchanged("0610101010".to_owned()),
                t: sea_orm::ActiveValue::Unchanged(true),
                pv: sea_orm::ActiveValue::Unchanged(true),
                two_fa: sea_orm::ActiveValue::Unchanged(true),
                lg: sea_orm::ActiveValue::Unchanged(Language::Fr),
                created_at: sea_orm::ActiveValue::Set(datetime_utc),
            }
        );
    } */

    /* {
        let user = Mutation::update_user_by_id(
            db,
            uuid!("4bcd5956-9e55-4c98-8ae6-ce03392083c4"),
            user_model::Model {
                id: uuid!("4bcd5956-9e55-4c98-8ae6-ce03392083c4"),
                av: None,
                f: String::from("Bob"),
                l: String::from("Labricole"),
                e: String::from("michel.boittout@gmail.com"),
                ph: String::from("0630303030"),
                t: true,
                pv: true,
                two_fa: true,
                lg: Language::Fr,
                created_at: Utc::now(),
            },
        )
        .await
        .unwrap();

        assert_eq!(
            user,
            user_model::Model {
                id: uuid!("4bcd5956-9e55-4c98-8ae6-ce03392083c4"),
                av: None,
                f: String::from("Bob"),
                l: String::from("Labricole"),
                e: String::from("michel.boittout@gmail.com"),
                ph: String::from("0630303030"),
                t: true,
                pv: true,
                two_fa: true,
                lg: Language::Fr,
                created_at: Utc::now(),
            }
        );
    } */

    {
        let result = UserMutation::delete_user(db, uuid!("4bcd5956-9e55-4c98-8ae6-ce03392083c4"))
            .await
            .expect("Cannot delete user.");

        assert_eq!(result.rows_affected, 1);
    }
}
