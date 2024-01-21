use actix_web::web::Data;
use async_trait::async_trait;
use chrono::Utc;
use entity::entities::two_fa_entity::two_fa_model;
use entity::entities::user_entity::user_model::Model as UserModel;
use sea_orm::{ActiveValue::Set, DatabaseConnection, DbErr};
use serde::{Deserialize, Serialize};
use service::mutation::two_fa_mutations::TwoFaMutation;
use tracing::error;

use crate::sms::send_auth_code_sms::{send_auth_code_sms, AuthCodeSmsData};
use rand::Rng;

#[derive(Debug, Serialize, Deserialize)]
pub struct RespCheckCode {
    pub valid: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RespCheckDeadLine {
    pub still_time: bool,
    pub time_left: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NumOfSending {
    pub sent: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SendingState {
    Sent = 0,
    NotSent = 1,
    AlreadySent = 2,
}

#[async_trait]
pub trait TwoFactorsAuth {
    fn get_number_of_sending(&self) -> i32;
    fn get_tries(&self) -> i32;
    fn get_exponent(&self) -> i32;
    fn get_deadline(&self) -> Option<i64>;

    fn generate_code(&self) -> String;
    fn check_code(&self, code: &String) -> RespCheckCode;
    async fn block_account<'a>(&'a self, db: &'a Data<DatabaseConnection>) -> i64;
    fn check_deadline(&self) -> RespCheckDeadLine;
    async fn send_code_to_pro(&self, user: UserModel, code: &String) -> SendingState;
    async fn reset_validation_system(&self, db: &Data<DatabaseConnection>);
    async fn reset_tries(&self, db: &Data<DatabaseConnection>) -> Result<(), DbErr>;
    async fn update_pro_by_remove_one_try(&self, db: &Data<DatabaseConnection>) -> i32;
    async fn update_pro_with_new_deadline(&self, db: &Data<DatabaseConnection>) -> i64;
    async fn update_two_fa_with_new_code(&self, code: &String, db: &Data<DatabaseConnection>);
    async fn update_two_fa_with_new_num_of_sending(&self, db: &Data<DatabaseConnection>) -> i32;
    fn new_deadline(&self, new_exponent: i32) -> i64;
    fn create_extra_time(&self, exponent: i32) -> i64;
}

#[async_trait]
impl TwoFactorsAuth for two_fa_model::Model {
    fn get_number_of_sending(&self) -> i32 {
        return self.s;
    }

    fn get_tries(&self) -> i32 {
        return self.t;
    }

    fn get_exponent(&self) -> i32 {
        return self.ex;
    }

    fn get_deadline(&self) -> Option<i64> {
        return self.up;
    }

    fn generate_code(&self) -> String {
        return (0..7)
            .map(|_| rand::thread_rng().gen_range(0..=9).to_string())
            .collect();
    }

    fn check_code(&self, code: &String) -> RespCheckCode {
        RespCheckCode {
            valid: self.c.as_ref().unwrap() == code,
        }
    }

    fn check_deadline(&self) -> RespCheckDeadLine {
        let now = Utc::now().timestamp_millis() as i64;
        let deadline = self.get_deadline();
        if deadline.is_none() {
            return RespCheckDeadLine {
                still_time: false,
                time_left: 0,
            };
        } else {
            return RespCheckDeadLine {
                still_time: deadline.unwrap() > now,
                time_left: deadline.unwrap() - now,
            };
        }
    }

    async fn send_code_to_pro(&self, user: UserModel, code: &String) -> SendingState {
        if self.get_number_of_sending() >= 2 {
            return SendingState::AlreadySent;
        } else {
            let data = AuthCodeSmsData {
                phone: user.ph.to_string(),
                first_name: user.f.to_string(),
                code: code.to_string(),
            };
            match send_auth_code_sms(data).await {
                Ok(_) => return SendingState::Sent,
                Err(_) => return SendingState::NotSent,
            }
        }
    }

    async fn reset_validation_system(&self, db: &Data<DatabaseConnection>) {
        let two_fa = two_fa_model::ActiveModel {
            id: Set(self.id.to_owned()),
            v_e: Set(false),
            t: Set(3),
            s: Set(0),
            c: Set(None),
            up: Set(None),
            ex: Set(0),
            v_ph: Set(false),
            user_id: Set(self.user_id.to_owned()),
        };

        match TwoFaMutation::update_two_fa(db, two_fa).await {
            Ok(_) => (),
            Err(err) => {
                error!("Failed to reset validation system: {}", err);
            }
        }
    }

    async fn reset_tries(&self, db: &Data<DatabaseConnection>) -> Result<(), DbErr> {
        if self.get_number_of_sending() < 2 && self.get_tries() > 0 {
            return Ok(());
        } else {
            let mut two_fa: two_fa_model::ActiveModel = self.to_owned().into();
            two_fa.t = Set(3);
            two_fa.s = Set(0);

            match TwoFaMutation::update_two_fa(db, two_fa).await {
                Ok(_) => return Ok(()),
                Err(err) => {
                    error!("Failed to reset tries: {}", err);
                    return Err(err);
                }
            }
        }
    }

    async fn update_pro_by_remove_one_try(&self, db: &Data<DatabaseConnection>) -> i32 {
        let tries = self.get_tries() - 1;
        let mut two_fa: two_fa_model::ActiveModel = self.clone().into();
        two_fa.t = Set(tries);

        match TwoFaMutation::update_two_fa(db, two_fa).await {
            Ok(_) => (),
            Err(err) => {
                error!("Failed to update two_fa by remove one try: {}", err);
            }
        }

        return tries;
    }

    async fn update_two_fa_with_new_code(&self, code: &String, db: &Data<DatabaseConnection>) {
        let mut two_fa: two_fa_model::ActiveModel = self.clone().into();
        two_fa.c = Set(Some(code.to_owned()));

        match TwoFaMutation::update_two_fa(db, two_fa).await {
            Ok(_) => (),
            Err(err) => {
                error!("Failed to update two_fa with new code: {}", err);
            }
        }
    }

    async fn update_two_fa_with_new_num_of_sending(&self, db: &Data<DatabaseConnection>) -> i32 {
        let num_of_sending = self.get_number_of_sending() + 1;
        let mut two_fa: two_fa_model::ActiveModel = self.clone().into();
        two_fa.s = Set(num_of_sending);

        match TwoFaMutation::update_two_fa(db, two_fa).await {
            Ok(_) => (),
            Err(err) => {
                error!("Failed to update two_fa with new num of sending: {}", err);
            }
        }

        return num_of_sending;
    }

    async fn block_account<'a>(&'a self, db: &'a Data<DatabaseConnection>) -> i64 {
        let deadline = self.update_pro_with_new_deadline(db).await;
        let now = Utc::now().timestamp_millis() as i64;
        return deadline - now;
    }

    async fn update_pro_with_new_deadline(&self, db: &Data<DatabaseConnection>) -> i64 {
        let new_exponent = self.get_exponent() + 1;
        let new_deadline = self.new_deadline(new_exponent);
        let mut two_fa: two_fa_model::ActiveModel = self.clone().into();
        two_fa.ex = Set(new_exponent);
        two_fa.up = Set(Some(new_deadline));

        match TwoFaMutation::update_two_fa(db, two_fa).await {
            Ok(_) => (),
            Err(err) => {
                error!("Failed to update two_fa with new num of sending: {}", err);
            }
        }

        return new_deadline;
    }

    fn new_deadline(&self, new_exponent: i32) -> i64 {
        let now = Utc::now().timestamp_millis() as i64;
        let extra_time = self.create_extra_time(new_exponent);
        return now + extra_time;
    }

    fn create_extra_time(&self, new_exponent: i32) -> i64 {
        if new_exponent == 0 {
            return 0;
        }

        let exponential_value = f64::powi(5.0, new_exponent);
        return (exponential_value * 60.0 * 1000.0) as i64;
    }
}
