use core::time::Duration;

use ockam_core::compat::sync::Arc;
use ockam_core::Result;
use ockam_vault::{VaultForSigning, VaultForVerifyingSignatures};

use crate::models::{Attributes, Credential, CredentialAndPurposeKey, CredentialData, Identifier};
use crate::utils::{add_seconds, now};
use crate::{IdentitiesCreation, PurposeKeyCreation};

/// Service for managing [`Credential`]s
pub struct CredentialsCreation {
    purpose_keys_creation: Arc<PurposeKeyCreation>,
    credential_vault: Arc<dyn VaultForSigning>,
    verifying_vault: Arc<dyn VaultForVerifyingSignatures>,
    identities_creation: Arc<IdentitiesCreation>,
}

impl CredentialsCreation {
    ///Constructor
    pub fn new(
        purpose_keys_creation: Arc<PurposeKeyCreation>,
        credential_vault: Arc<dyn VaultForSigning>,
        verifying_vault: Arc<dyn VaultForVerifyingSignatures>,
        identities_creation: Arc<IdentitiesCreation>,
    ) -> Self {
        Self {
            purpose_keys_creation,
            verifying_vault,
            credential_vault,
            identities_creation,
        }
    }
}

impl CredentialsCreation {
    /// Issue a [`Credential`]
    pub async fn issue_credential(
        &self,
        issuer: &Identifier,
        subject: &Identifier,
        subject_attributes: Attributes,
        ttl: Duration,
    ) -> Result<CredentialAndPurposeKey> {
        // TODO: Allow manual PurposeKey management
        let issuer_purpose_key = self
            .purpose_keys_creation
            .get_or_create_credential_purpose_key(issuer)
            .await?;

        let subject_identity = self.identities_creation.get_identity(subject).await?;

        let created_at = now()?;
        let expires_at = add_seconds(&created_at, ttl.as_secs());

        let credential_data = CredentialData {
            subject: Some(subject.clone()),
            subject_latest_change_hash: Some(subject_identity.latest_change_hash()?.clone()),
            subject_attributes,
            created_at,
            expires_at,
        };
        let credential_data = minicbor::to_vec(credential_data)?;

        let versioned_data = Credential::create_versioned_data(credential_data);
        let versioned_data = minicbor::to_vec(&versioned_data)?;

        let versioned_data_hash = self.verifying_vault.sha256(&versioned_data).await?;

        let signature = self
            .credential_vault
            .sign(issuer_purpose_key.key(), &versioned_data_hash.0)
            .await?;
        let signature = signature.into();

        let credential = Credential {
            data: versioned_data,
            signature,
        };

        let res = CredentialAndPurposeKey {
            credential,
            purpose_key_attestation: issuer_purpose_key.attestation().clone(),
        };

        Ok(res)
    }
}
