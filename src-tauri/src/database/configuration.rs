use diesel::prelude::*;

use super::{establish_connection, models, schema};

use crate::model::configuration::SettingName;

pub fn reset_configuration() -> () {
    use schema::configuration;

    let connection = establish_connection();
    diesel::delete(configuration::table)
        .execute(&connection)
        .expect("Could not delete table contents");
}

pub fn increment_compactions() -> () {
    use schema::configuration;

    let connection = establish_connection();
    let compactions = get_compactions() + 1;

    diesel::update(configuration::table.find(SettingName::Compactions as i32))
        .set(configuration::setting_value.eq(compactions.to_string()))
        .execute(&connection)
        .expect("Could not increment compactions");
}

pub fn increment_condensations() -> () {
    use schema::configuration;

    let connection = establish_connection();
    let condensations = get_condensations() + 1;

    diesel::update(configuration::table.find(SettingName::Condensations as i32))
        .set(configuration::setting_value.eq(condensations.to_string()))
        .execute(&connection)
        .expect("Could not increment condensations");
}

pub fn increment_partition_consecutive_number() -> () {
    use schema::configuration;

    let connection = establish_connection();
    let last_consecutive_number = get_partition_consecutive_number() + 1;

    diesel::update(configuration::table.find(SettingName::PartitionConsecutiveNumber as i32))
        .set(configuration::setting_value.eq(last_consecutive_number.to_string()))
        .execute(&connection)
        .expect("Could not increment partition consecutive number");
}

pub fn get_compactions() -> i32 {
    use schema::configuration;

    let connection = establish_connection();

    let compactions = get_configuration_value(SettingName::Compactions);
    if compactions.is_err() {
        diesel::insert_into(configuration::table)
            .values(&models::Configuration {
                setting_id: SettingName::Compactions as i32,
                setting_value: 0.to_string(),
            })
            .execute(&connection)
            .expect("Could not create compactions number");
    }
    get_configuration_value(SettingName::Compactions)
        .unwrap()
        .setting_value
        .parse::<i32>()
        .unwrap()
}

pub fn get_condensations() -> i32 {
    use schema::configuration;

    let connection = establish_connection();

    let condensations = get_configuration_value(SettingName::Condensations);

    if condensations.is_err() {
        diesel::insert_into(configuration::table)
            .values(&models::Configuration {
                setting_id: SettingName::Condensations as i32,
                setting_value: 0.to_string(),
            })
            .execute(&connection)
            .expect("Could not create condensation number");
    }

    get_configuration_value(SettingName::Condensations)
        .unwrap()
        .setting_value
        .parse::<i32>()
        .unwrap()
}

pub fn get_partition_consecutive_number() -> i32 {
    use schema::configuration;

    let connection = establish_connection();

    let partition_consecutive_number =
        get_configuration_value(SettingName::PartitionConsecutiveNumber);

    if partition_consecutive_number.is_err() {
        diesel::insert_into(configuration::table)
            .values(&models::Configuration {
                setting_id: SettingName::PartitionConsecutiveNumber as i32,
                setting_value: 1.to_string(),
            })
            .execute(&connection)
            .expect("Could not create partition consecutive number");
    }
    get_configuration_value(SettingName::PartitionConsecutiveNumber)
        .unwrap()
        .setting_value
        .parse::<i32>()
        .unwrap()
}

pub fn set_memory_size(size: i32) -> () {
    use schema::configuration;

    let connection = establish_connection();

    // The configuration has not been set yet.
    if configuration::table
        .filter(configuration::setting_id.eq(SettingName::MemorySize as i32))
        .first::<models::Configuration>(&connection)
        .is_err()
    {
        diesel::insert_into(configuration::table)
            .values(&models::Configuration {
                setting_id: SettingName::MemorySize as i32,
                setting_value: size.to_string(),
            })
            .execute(&connection)
            .expect("Could not init configuration.");
    } else {
        diesel::update(configuration::table.find(SettingName::MemorySize as i32))
            .set(configuration::setting_value.eq(size.to_string()))
            .execute(&connection)
            .expect("Could not update memory size");
    }
}

pub fn get_memory_size() -> i32 {
    return get_configuration_value(SettingName::MemorySize)
        .unwrap()
        .setting_value
        .parse::<i32>()
        .unwrap();
}

fn get_configuration_value(value: SettingName) -> QueryResult<models::Configuration> {
    use schema::configuration;

    let connection = establish_connection();
    configuration::table
        .find(value as i32)
        .first::<models::Configuration>(&connection)
}
