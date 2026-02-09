use anyhow::{Context, Result};
use playwright::Playwright;
use playwright::api::{Browser, ElementHandle, Page};
use playwright::api::frame::FrameState;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, error, warn, debug};

use crate::models::LogGame;
use crate::repository::GameRepository;

// Configuration constants
const DEFAULT_NAVIGATION_TIMEOUT_MS: f64 = 45_000.0; // 45 seconds
const DEFAULT_ELEMENT_TIMEOUT_MS: f64 = 30_000.0;   // 30 seconds
const SAFETY_SLEEP_MS: u64 = 2000;                   // 2 seconds safety buffer
const RETRY_DELAY_MS: u64 = 1000;                    // 1 second between retries
const MAX_RETRIES: u32 = 3;

pub struct ScraperService {
    playwright: Playwright,
    repository: GameRepository,
}

impl ScraperService {
    pub async fn new(repository: GameRepository) -> Result<Self> {
        let playwright = Playwright::initialize().await?;
        
        Ok(Self {
            playwright,
            repository,
        })
    }

    /// Create browser instance with headless configuration
    async fn create_browser(&self, headless: bool) -> Result<Browser> {
        let chromium = self.playwright.chromium();
        
        let args = vec![
            "--start-maximized".to_string(),
            "--no-sandbox".to_string(),
            "--disable-infobars".to_string(),
            "--disable-dev-shm-usage".to_string(),
            "--disable-browser-side-navigation".to_string(),
            "--disable-gpu".to_string(),
            "--ignore-certificate-errors".to_string(),
        ];

        let browser = chromium
            .launcher()
            .headless(headless)
            .args(&args)
            .launch()
            .await?;

        Ok(browser)
    }

    /// Wait for element dengan timeout yang proper
    /// Menunggu sampai element benar-benar visible, tidak hanya ada di DOM
    async fn wait_for_element(
        page: &Page,
        selector: &str,
        timeout_ms: f64,
    ) -> Result<ElementHandle> {
        debug!("Waiting for element: {}", selector);
        
        let element = page
            .wait_for_selector_builder(selector)
            .timeout(timeout_ms)
                        .state(FrameState::Visible)
            .wait_for_selector()
            .await
            .with_context(|| format!("Timeout {}ms menunggu element: {}", timeout_ms, selector))?
            .ok_or_else(|| anyhow::anyhow!("Element tidak ditemukan: {}", selector))?;
        
        Ok(element)
    }

    /// Navigate ke URL dan tunggu sampai key element muncul
    async fn navigate_and_wait(
        page: &Page,
        url: &str,
        wait_selector: &str,
    ) -> Result<()> {
        debug!("Navigating to: {}", url);
        
        page.goto_builder(url)
            .timeout(DEFAULT_NAVIGATION_TIMEOUT_MS)
            .goto()
            .await
            .with_context(|| format!("Gagal navigate ke: {}", url))?;
        
        // Wait for key element yang menandakan page sudah ready
        Self::wait_for_element(page, wait_selector, DEFAULT_ELEMENT_TIMEOUT_MS).await?;
        
        // Safety sleep - tambahan buffer untuk JavaScript yang masih running
        sleep(Duration::from_millis(SAFETY_SLEEP_MS)).await;
        
        Ok(())
    }

    /// Try wait for element dengan fallback - returns None jika tidak ditemukan
    async fn try_wait_for_element(
        page: &Page,
        selector: &str,
        timeout_ms: f64,
    ) -> Option<ElementHandle> {
        match page
            .wait_for_selector_builder(selector)
            .timeout(timeout_ms)
                        .state(FrameState::Visible)
            .wait_for_selector()
            .await
        {
            Ok(Some(element)) => Some(element),
            _ => None,
        }
    }

    /// Update result for MQ games (YoungToto provider)
    pub async fn update_result_per_game_mq(&self, count_grab_data: i32) -> Result<()> {
        info!("Starting MQ game result update");
        
        let browser = self.create_browser(false).await?;
        let context = browser.context_builder().build().await?;
        let page = context.new_page().await?;

        // Get URL configuration
        let url_header = self.repository.get_link_header().await?
            .context("URL header not found")?;
        
        let list_url_details = self.repository.get_link_details_mq().await?;
        
        // Open home page first - wait for body to ensure basic page load
        let url_home = url_header.link_game.replace("/pasaran/", "");
        page.goto_builder(&url_home)
            .timeout(DEFAULT_NAVIGATION_TIMEOUT_MS)
            .goto()
            .await
            .with_context(|| "Gagal membuka home page")?;
        
        // Wait for page body and safety sleep
        let _ = Self::try_wait_for_element(&page, "body", DEFAULT_ELEMENT_TIMEOUT_MS).await;
        sleep(Duration::from_millis(SAFETY_SLEEP_MS)).await;

        // Process each game
        for url_detail in list_url_details {
            let game_code_ref = url_detail.game_code.clone();
            match self.process_mq_game(&page, &url_header.link_game, &url_detail.link_game, count_grab_data).await {
                Ok(_) => {
                    info!("Successfully processed game: {:?}", game_code_ref);
                }
                Err(e) => {
                    error!("Error processing game {:?}: {}", game_code_ref, e);
                }
            }
        }

        browser.close().await?;
        Ok(())
    }

    /// Process individual MQ game
    async fn process_mq_game(
        &self,
        page: &playwright::api::Page,
        url_header: &str,
        url_detail: &str,
        count_grab_data: i32,
    ) -> Result<()> {
        let mut url_result = format!("{}{}", url_header, url_detail);
        url_result = url_result.replace("?per_page=0", "");
        
        // Key selector untuk MQ page - header yang berisi game code
        let key_selector = "#pageContent > ul > li:nth-child(7) > span.title.text-bold";
        
        info!("Navigating to MQ game page: {}", url_result);
        
        page.goto_builder(&url_result)
            .timeout(DEFAULT_NAVIGATION_TIMEOUT_MS)
            .goto()
            .await
            .with_context(|| format!("Gagal navigate ke game page: {}", url_result))?;
        
        // Wait for key element yang menandakan data sudah ready
        Self::wait_for_element(&page, key_selector, DEFAULT_ELEMENT_TIMEOUT_MS)
            .await
            .with_context(|| "Page belum selesai load - element header tidak ditemukan")?;
        
        // Safety sleep untuk JavaScript yang mungkin masih running
        sleep(Duration::from_millis(SAFETY_SLEEP_MS)).await;

        self.grab_data_mq(&page, count_grab_data).await?;
        
        Ok(())
    }

    async fn grab_data_mq(&self, page: &Page, count_grab_data: i32) -> Result<()> {
        // Get game code from header - element sudah di-wait di process_mq_game
        let css_header = "#pageContent > ul > li:nth-child(7) > span.title.text-bold";
        let header_elem = Self::wait_for_element(page, css_header, DEFAULT_ELEMENT_TIMEOUT_MS)
            .await
            .with_context(|| "Header element untuk game code tidak ditemukan")?;
        let header_text = header_elem.inner_text().await?;
        
        let game_code = header_text
            .split('-')
            .next()
            .unwrap_or("")
            .trim()
            .to_string();

        info!("Grabbing data for game: {}", game_code);

        // Get last month logs for comparison
        let last_logs = self.repository.get_last_month_log_games(&game_code).await?;

        // Iterate through results (from bottom to top)
        for i in (2..=count_grab_data).rev() {
            match self.extract_log_entry(page, i, &game_code, &last_logs).await {
                Ok(Some(_)) => {
                    // Log saved successfully
                }
                Ok(None) => {
                    // Log already exists, skipped
                }
                Err(e) => {
                    warn!("Error extracting log at position {}: {}", i, e);
                }
            }
        }

        Ok(())
    }

    /// Extract single log entry from page
    async fn extract_log_entry(
        &self,
        page: &playwright::api::Page,
        position: i32,
        game_code: &str,
        last_logs: &[LogGame],
    ) -> Result<Option<LogGame>> {
        let css_game_periode = format!(
            "#pageContent > ul > li:nth-child({}) > span.title.text-bold",
            position
        );
        let css_date = format!(
            "#pageContent > ul > li:nth-child({}) > span.secondary-content.read-text",
            position
        );
        let css_result = format!("#pageContent > ul > li:nth-child({}) > p", position);

        // Extract data
        let game_periode_elem = page.query_selector(&css_game_periode).await?;
        let date_elem = page.query_selector(&css_date).await?;
        let result_elem = page.query_selector(&css_result).await?;
        
        if game_periode_elem.is_none() || date_elem.is_none() || result_elem.is_none() {
            return Ok(None);
        }
        
        let game_periode_text = game_periode_elem.unwrap().inner_text().await?;
        let date_text = date_elem.unwrap().inner_text().await?;
        let result_text = result_elem.unwrap().inner_text().await?;

        sleep(Duration::from_millis(500)).await;

        // Parse periode
        let periode_str = game_periode_text
            .split('-')
            .nth(1)
            .unwrap_or("0")
            .trim();
        let periode: i32 = periode_str.parse().unwrap_or(0);

        let log_result = result_text.trim().to_string();
        let date_result = date_text.trim().to_string();

        // Check if log already exists
        let exists = last_logs.iter().any(|l| {
            l.game_code == game_code && l.periode == periode && l.log_result == log_result
        });

        if exists {
            return Ok(None);
        }

        // Create and save new log
        let log = LogGame::new(
            game_code.to_string(),
            periode,
            log_result.clone(),
            date_result.clone(),
        );

        self.repository.save_log(&log).await?;

        // Update master game
        use crate::models::MasterGame;
        let master = MasterGame::new(
            game_code.to_string(),
            periode,
            log_result,
            date_result,
        );
        
        self.repository.update_master_game(&master).await?;

        Ok(Some(log))
    }

    /// Update result for regular games (non-MQ)
    pub async fn update_result_per_game(&self, count_grab_data: i32) -> Result<()> {
        info!("Starting regular game result update");
        
        let browser = self.create_browser(false).await?;
        let context = browser.context_builder().build().await?;
        let page = context.new_page().await?;

        // Get URL configuration
        let url_header = self.repository.get_link_header().await?
            .context("URL header not found")?;
        
        let list_url_details = self.repository.get_link_details().await?;
        
        // Open home page first - wait for body to ensure basic page load
        let url_home = url_header.link_game.replace("/pasaran/", "");
        page.goto_builder(&url_home)
            .timeout(DEFAULT_NAVIGATION_TIMEOUT_MS)
            .goto()
            .await
            .with_context(|| "Gagal membuka home page")?;
        
        // Wait for page body and safety sleep
        let _ = Self::try_wait_for_element(&page, "body", DEFAULT_ELEMENT_TIMEOUT_MS).await;
        sleep(Duration::from_millis(SAFETY_SLEEP_MS)).await;

        // Process each game
        for url_detail in list_url_details {
            let game_code_ref = url_detail.game_code.clone();
            match self.process_regular_game(&page, &url_header.link_game, &url_detail.link_game, count_grab_data).await {
                Ok(_) => {
                    info!("Successfully processed game: {:?}", game_code_ref);
                }
                Err(e) => {
                    error!("Error processing game {:?}: {}", game_code_ref, e);
                }
            }
        }

        browser.close().await?;
        Ok(())
    }

    /// Process individual regular game
    async fn process_regular_game(
        &self,
        page: &playwright::api::Page,
        url_header: &str,
        url_detail: &str,
        count_grab_data: i32,
    ) -> Result<()> {
        // Key selector untuk regular page - table yang berisi data
        let table_selector = "#pasaran > div > table > tbody";
        let url_result = format!("{}{}0", url_header, url_detail);
        
        info!("Navigating to regular game page: {}", url_result);
        
        page.goto_builder(&url_result)
            .timeout(DEFAULT_NAVIGATION_TIMEOUT_MS)
            .goto()
            .await
            .with_context(|| format!("Gagal navigate ke: {}", url_result))?;
        
        // Wait for table yang menandakan data sudah ready
        Self::wait_for_element(&page, table_selector, DEFAULT_ELEMENT_TIMEOUT_MS)
            .await
            .with_context(|| "Page belum selesai load - table tidak ditemukan")?;
        
        // Safety sleep
        sleep(Duration::from_millis(SAFETY_SLEEP_MS)).await;

        self.grab_data_regular(&page, count_grab_data).await?;
        
        Ok(())
    }

    async fn grab_data_regular(&self, page: &Page, count_grab_data: i32) -> Result<()> {
        // Regular games use table format - element sudah di-wait di process_regular_game
        let table_selector = "#pasaran > div > table > tbody";
        
        // Re-verify table exists
        Self::wait_for_element(page, table_selector, DEFAULT_ELEMENT_TIMEOUT_MS)
            .await
            .with_context(|| "Table element tidak ditemukan")?;
        
        // Count rows
        let rows = page.query_selector_all(&format!("{} > tr", table_selector)).await?;
        let total_rows = rows.len().min(count_grab_data as usize);

        info!("Found {} rows to process", total_rows);

        for i in 1..=total_rows {
            match self.extract_table_log_entry(page, i as i32, &table_selector).await {
                Ok(Some(_)) => {
                    // Log saved successfully
                }
                Ok(None) => {
                    // Log already exists, skipped
                }
                Err(e) => {
                    warn!("Error extracting table log at row {}: {}", i, e);
                }
            }
        }

        Ok(())
    }

    /// Extract log entry from table format
    async fn extract_table_log_entry(
        &self,
        page: &playwright::api::Page,
        row: i32,
        table_selector: &str,
    ) -> Result<Option<LogGame>> {
        // Table format: | Periode | Date | Result |
        let css_periode = format!("{} > tr:nth-child({}) > td:nth-child(1)", table_selector, row);
        let css_date = format!("{} > tr:nth-child({}) > td:nth-child(2)", table_selector, row);
        let css_result = format!("{} > tr:nth-child({}) > td:nth-child(3)", table_selector, row);

        let periode_elem = page.query_selector(&css_periode).await?;
        let date_elem = page.query_selector(&css_date).await?;
        let result_elem = page.query_selector(&css_result).await?;
        
        if periode_elem.is_none() || date_elem.is_none() || result_elem.is_none() {
            return Ok(None);
        }
        
        let periode_text = periode_elem.unwrap().inner_text().await?;
        let date_text = date_elem.unwrap().inner_text().await?;
        let result_text = result_elem.unwrap().inner_text().await?;

        // Parse data
        let parts: Vec<&str> = periode_text.split('-').collect();
        if parts.len() < 2 {
            return Ok(None);
        }

        let game_code = parts[0].trim().to_string();
        let periode: i32 = parts[1].trim().parse().unwrap_or(0);
        let log_result = result_text.trim().to_string();
        let date_result = date_text.trim().to_string();

        // Check if exists
        let existing = self.repository
            .get_log_by_game_code_and_periode(&game_code, periode)
            .await?;

        if existing.is_some() {
            return Ok(None);
        }

        // Create and save
        let log = LogGame::new(game_code.clone(), periode, log_result.clone(), date_result.clone());
        self.repository.save_log(&log).await?;

        // Update master
        use crate::models::MasterGame;
        let master = MasterGame::new(game_code, periode, log_result, date_result);
        self.repository.update_master_game(&master).await?;

        Ok(Some(log))
    }

    /// Correct missing logs
    pub async fn correct_logs(&self) -> Result<()> {
        info!("Starting log correction");
        
        let browser = self.create_browser(true).await?; // headless
        let context = browser.context_builder().build().await?;
        let page = context.new_page().await?;

        let all_games = self.repository.get_all_master_games().await?;

        for game in all_games {
            if game.game_code.contains("XX") {
                continue;
            }

            let logs = self.repository.get_logs_by_game_code(&game.game_code).await?;
            if logs.is_empty() {
                continue;
            }

            let missing = self.find_missing_periodes(&logs);
            
            if !missing.is_empty() {
                info!("Found {} missing periodes for {}", missing.len(), game.game_code);
                
                for (_periode, page_num) in missing {
                    if page_num < 1300 {
                        self.update_log_corrected(&page, &game.game_code, page_num).await?;
                    }
                }
            }
        }

        browser.close().await?;
        Ok(())
    }

    fn find_missing_periodes(&self, logs: &[LogGame]) -> Vec<(i32, i32)> {
        let mut missing = Vec::new();
        let existing: std::collections::HashSet<i32> = logs.iter().map(|l| l.periode).collect();

        for (i, log) in logs.iter().enumerate() {
            let target_periode = log.periode - i as i32;
            if !existing.contains(&target_periode) {
                let page_number = (i / 50) * 50;
                missing.push((target_periode, page_number as i32));
            }
        }

        missing
    }

    async fn update_log_corrected(&self, page: &Page, game_code: &str, page_number: i32) -> Result<()> {
        let url_header = self.repository.get_link_header().await?
            .context("URL header not found")?;
        
        let link_details = self.repository.get_link_details().await?;
        let url_detail = link_details.iter()
            .find(|d| d.game_code.as_deref() == Some(game_code))
            .context("Game code not found in link details")?;

        let url = format!("{}{}{}", url_header.link_game, url_detail.link_game, page_number);
        let table_selector = "#pasaran > div > table > tbody";
        
        debug!("Correcting logs from: {}", url);
        
        page.goto_builder(&url)
            .timeout(DEFAULT_NAVIGATION_TIMEOUT_MS)
            .goto()
            .await
            .with_context(|| format!("Gagal navigate ke: {}", url))?;
        
        // Wait for table dan safety sleep
        Self::wait_for_element(page, table_selector, DEFAULT_ELEMENT_TIMEOUT_MS)
            .await
            .with_context(|| "Table tidak ditemukan untuk log correction")?;
        sleep(Duration::from_millis(SAFETY_SLEEP_MS)).await;

        self.grab_data_regular(page, 50).await?;

        Ok(())
    }
}