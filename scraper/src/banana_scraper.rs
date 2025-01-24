use log::{debug, warn};
use reqwest::{Client, ClientBuilder};
use serde::Deserialize;
use thiserror::Error;

use crate::{
    types::ModId,
    utils::{
        FuzzySearchMatchRes, fuzzy_search_strings_and_return_one_or_many_depending_on_perfect_match,
    },
};

pub type BananaScraperResult<T> = Result<T, BananaScraperError>;

static BANANA_ROOT: &str = "https://gamebanana.com";

/// I think this is hard coded into the web interface and can't be changed?
const NUM_SEARCH_RESULTS_PER_PAGE: usize = 15;

#[derive(Debug, Error)]
pub enum BananaScraperError {
    #[error(
        "Unable to find the mod \"{0}\". Make sure that the mod name is an exact match (including case) to the mod name on GameBanana."
    )]
    ModNameNotFound(String),

    #[error("The mod variant {0} was not found for the mod {1}.")]
    ModVariantDoesNotFound(String, String),

    #[error(transparent)]
    JsonDeserializationError(#[from] serde_json::Error),

    #[error("Error during Reqwest client initialization")]
    ClientError(#[from] reqwest::Error),
}

#[derive(Debug)]
pub struct ScrapedBananaModData {
    name: String,
    version: Option<String>,
    variant_download_artifact: Vec<u8>,
}

#[derive(Debug)]
pub struct BananaClient {
    client: Client,
}

impl BananaClient {
    pub fn new() -> BananaScraperResult<Self> {
        let client = ClientBuilder::default().build()?;

        Ok(Self { client })
    }

    async fn get_mod_page_for_mod_id(&self, name: &str) -> BananaScraperResult<String> {
        todo!()
    }

    pub async fn resolve_mod_name(&self, name: &str) -> BananaScraperResult<ModId> {
        debug!("Resolving mod name \"{}\" to it's ID...", name);

        let search_req = format!(
            "{}/apiv11/Util/Search/Results?_sModelName=Mod&_sOrder=best_match&_idGameRow=6498&_sSearchString={}&_csvFields=name&_nPage=1",
            BANANA_ROOT, name
        );
        let search_resp: SearchResp =
            serde_json::from_str(&self.client.get(search_req).send().await?.text().await?)?;

        // We are going to enforce that searching by a name MUST match the name of the mod exactly, including case.
        match search_resp
            .a_records
            .into_iter()
            .find(|item| item.s_name == name)
        {
            Some(entry) => Ok(entry.id_row),
            None => {
                if search_resp.a_meta_data.n_record_count > NUM_SEARCH_RESULTS_PER_PAGE {
                    warn!(
                        "Unable to find an exact mod name match for \"{}\" on GameBanana. We only searched through {} entries but {} were matched by the search in general. Make sure that the name of the mod is entered exactly the way it's spelled (including case) on GameBanana.",
                        name, NUM_SEARCH_RESULTS_PER_PAGE, search_resp.a_meta_data.n_record_count
                    );
                }

                Err(BananaScraperError::ModNameNotFound(name.to_string()))
            }
        }
    }

    pub async fn download_mod_variant(
        &self,
        id: ModId,
        variant_name: &str,
    ) -> BananaScraperResult<ScrapedBananaModData> {
        debug!("Downloading mod variant {} - {}...", id, variant_name);

        let mod_page_req = format!("{}/apiv11/Mod/{}/ProfilePage", BANANA_ROOT, id);
        let mod_page_resp: ModPageResp =
            serde_json::from_str(&self.client.get(mod_page_req).send().await?.text().await?)?;

        // We're not going to require an exact match here, but will use fuzzy matching instead.
        let mod_file_names = mod_page_resp
            .a_files
            .iter()
            .map(|file| file.s_file.as_str())
            .collect::<Vec<_>>();

        let match_idx = match fuzzy_search_strings_and_return_one_or_many_depending_on_perfect_match(
            &mod_file_names,
            variant_name,
        ) {
            FuzzySearchMatchRes::Perfect(idx) => idx,
            FuzzySearchMatchRes::Multiple(vec) => todo!(),
            FuzzySearchMatchRes::None => {
                return Err(BananaScraperError::ModVariantDoesNotFound(
                    variant_name.to_string(),
                    mod_page_resp.s_Name,
                ));
            }
        };

        todo!()
    }
}

#[derive(Debug, Deserialize)]
struct SearchResp {
    #[serde(rename = "_aMetadata")]
    a_meta_data: SearchMetaData,

    #[serde(rename = "_aRecords")]
    a_records: Vec<SearchRespItem>,
}

#[derive(Debug, Deserialize)]
struct SearchMetaData {
    #[serde(rename = "_nRecordCount")]
    n_record_count: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchRespItem {
    #[serde(rename = "_idRow")]
    id_row: ModId,

    #[serde(rename = "_sName")]
    s_name: String,
}

#[derive(Debug, Deserialize)]
struct ModPageResp {
    #[serde(rename = "_sName")]
    s_Name: String,

    #[serde(rename = "_aFiles")]
    a_files: Vec<ModDownloadEntries>,
}

#[derive(Debug, Deserialize)]
struct ModDownloadEntries {
    #[serde(rename = "_sFile")]
    s_file: String,

    #[serde(rename = "_tsDateAdded")]
    ts_date_added: u64,

    #[serde(rename = "_sDownloadUrl")]
    s_download_url: String,

    #[serde(rename = "_sMd5Checksum")]
    s_md5_checksum: String,
}
