use oceaniam_database::config::application::{
    ApplicationConfiguration, Argon2Configuration as DbArgon2Configuration, AuthConfiguration,
    PasswordConfiguration, RegistrationConfiguration, TokenConfiguration,
};
use oceaniam_vo::applications::{
    ApplicationConfigurationVO, Argon2Configuration, AuthConfigurationVO, PasswordConfigurationVO,
    RegistrationConfigurationVO, TokenConfigurationVO,
};

pub fn token_configuration_to_vo(config: TokenConfiguration) -> TokenConfigurationVO {
    let TokenConfiguration { issuer, audience } = config;
    TokenConfigurationVO { issuer, audience }
}

pub fn argon2_configuration_to_vo(config: DbArgon2Configuration) -> Argon2Configuration {
    let DbArgon2Configuration {
        m_cost,
        t_cost,
        p_cost,
    } = config;
    Argon2Configuration {
        m_cost,
        t_cost,
        p_cost,
    }
}

pub fn password_configuration_to_vo(config: PasswordConfiguration) -> PasswordConfigurationVO {
    let PasswordConfiguration { argon2 } = config;
    PasswordConfigurationVO {
        argon2: argon2_configuration_to_vo(argon2),
    }
}

pub fn auth_configuration_to_vo(config: AuthConfiguration) -> AuthConfigurationVO {
    let AuthConfiguration { token, password } = config;
    AuthConfigurationVO {
        token: token_configuration_to_vo(token),
        password: password_configuration_to_vo(password),
    }
}

pub fn registration_configuration_to_vo(
    config: RegistrationConfiguration,
) -> RegistrationConfigurationVO {
    let RegistrationConfiguration { enabled } = config;
    RegistrationConfigurationVO { enabled }
}

pub fn application_configuration_to_vo(
    config: ApplicationConfiguration,
) -> ApplicationConfigurationVO {
    let ApplicationConfiguration { auth, registration } = config;
    ApplicationConfigurationVO {
        auth: auth_configuration_to_vo(auth),
        registration: registration_configuration_to_vo(registration),
    }
}
