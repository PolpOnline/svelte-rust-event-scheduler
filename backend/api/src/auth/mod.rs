#[derive(Clone, Debug, Deserialize, Serialize)]
struct RevocationEndpointProviderMetadata {
    revocation_endpoint: String,
}

impl AdditionalProviderMetadata for RevocationEndpointProviderMetadata {}

type GoogleProviderMetadata = ProviderMetadata<
    RevocationEndpointProviderMetadata,
    CoreAuthDisplay,
    CoreClientAuthMethod,
    CoreClaimName,
    CoreClaimType,
    CoreGrantType,
    CoreJweContentEncryptionAlgorithm,
    CoreJweKeyManagementAlgorithm,
    CoreJwsSigningAlgorithm,
    CoreJsonWebKeyType,
    CoreJsonWebKeyUse,
    CoreJsonWebKey,
    CoreResponseMode,
    CoreResponseType,
    CoreSubjectIdentifierType,
>;

pub async fn build_google_oauth_client(secret: &SecretStore, redirect_uri: String) -> CoreClient {
    let client_id = secret
        .get("GOOGLE_CLIENT_ID")
        .expect("Missing GOOGLE_CLIENT_ID!");
    let client_id = ClientId::new(client_id.to_string());
    let client_secret = secret
        .get("GOOGLE_CLIENT_SECRET")
        .expect("Missing GOOGLE_CLIENT_SECRET!");
    let client_secret = ClientSecret::new(client_secret.to_string());
    let issuer_url =
        IssuerUrl::new("https://accounts.google.com".to_string()).expect("Invalid issuer URL");

    let provider_metadata = GoogleProviderMetadata::discover_async(issuer_url, async_http_client)
        .await
        .expect("Failed to discover Google provider metadata");

    let revocation_endpoint = provider_metadata
        .additional_metadata()
        .revocation_endpoint
        .clone();

    CoreClient::from_provider_metadata(provider_metadata, client_id, Some(client_secret))
        .set_redirect_uri(RedirectUrl::new(redirect_uri).expect("Invalid redirect URL"))
        .set_revocation_uri(
            RevocationUrl::new(revocation_endpoint).expect("Invalid revocation endpoint URL"),
        )
}
