use game_models::*;
use game_repository::{
    MasterGameRepository,
    MemberGameRepository,
    SiteMasterRepository,
    SetupLinkGameRepository,
};
use crate::errors::GameResult;

pub struct SetupService {
    master_repo: MasterGameRepository,
    member_repo: MemberGameRepository,
    site_repo: SiteMasterRepository,
    link_repo: SetupLinkGameRepository,
}

impl SetupService {
    pub fn new(
        master_repo: MasterGameRepository,
        member_repo: MemberGameRepository,
        site_repo: SiteMasterRepository,
        link_repo: SetupLinkGameRepository,
    ) -> Self {
        Self {
            master_repo,
            member_repo,
            site_repo,
            link_repo,
        }
    }

    // --- Master Game ---
    pub async fn get_all_master_games(&self) -> GameResult<Vec<MasterGame>> {
        self.master_repo.find_all().await.map_err(Into::into)
    }

    pub async fn create_master_game(&self, data: CreateMasterGame) -> GameResult<MasterGame> {
        self.master_repo.create(&data).await.map_err(Into::into)
    }

    pub async fn update_master_game(&self, data: UpdateMasterGame) -> GameResult<MasterGame> {
        self.master_repo.update(&data).await.map_err(Into::into)
    }

    pub async fn delete_master_game(&self, id: i64) -> GameResult<bool> {
        self.master_repo.delete(id).await.map_err(Into::into)
    }

    // --- Member Game ---
    pub async fn get_all_member_games(&self) -> GameResult<Vec<MemberGame>> {
        self.member_repo.find_all().await.map_err(Into::into)
    }

    pub async fn create_member_game(&self, data: CreateMemberGame) -> GameResult<MemberGame> {
        self.member_repo.create(&data).await.map_err(Into::into)
    }

    pub async fn update_member_game(&self, data: UpdateMemberGame) -> GameResult<MemberGame> {
        self.member_repo.update(&data).await.map_err(Into::into)
    }

    pub async fn delete_member_game(&self, id: i64) -> GameResult<bool> {
        self.member_repo.delete(id).await.map_err(Into::into)
    }

    // --- Site Master ---
    pub async fn get_all_site_masters(&self) -> GameResult<Vec<SiteMaster>> {
        self.site_repo.find_all().await.map_err(Into::into)
    }

    pub async fn create_site_master(&self, data: CreateSiteMaster) -> GameResult<SiteMaster> {
        self.site_repo.create(&data).await.map_err(Into::into)
    }

    pub async fn update_site_master(&self, data: UpdateSiteMaster) -> GameResult<SiteMaster> {
        self.site_repo.update(&data).await.map_err(Into::into)
    }

    pub async fn delete_site_master(&self, id: i64) -> GameResult<bool> {
        self.site_repo.delete(id).await.map_err(Into::into)
    }

    // --- Setup Link Game ---
    pub async fn get_all_link_games(&self) -> GameResult<Vec<SetupLinkGame>> {
        self.link_repo.find_all().await.map_err(Into::into)
    }

    pub async fn create_link_game(&self, data: CreateSetupLinkGame) -> GameResult<SetupLinkGame> {
        self.link_repo.create(&data).await.map_err(Into::into)
    }

    pub async fn update_link_game(&self, data: UpdateSetupLinkGame) -> GameResult<SetupLinkGame> {
        self.link_repo.update(&data).await.map_err(Into::into)
    }

    pub async fn delete_link_game(&self, id: i64) -> GameResult<bool> {
        self.link_repo.delete(id).await.map_err(Into::into)
    }
}
