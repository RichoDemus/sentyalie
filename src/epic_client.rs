use crate::{Game, Platform};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use reqwest::get;

pub(crate) async fn get_free_games() -> Vec<Game> {
    let response = get("https://store-site-backend-static.ak.epicgames.com/freeGamesPromotions")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    parse_and_filter(response.as_str())
}

fn parse_and_filter(json: &str) -> Vec<Game> {
    let response: Response = serde_json::from_str(json).unwrap();

    let now = Utc::now(); // todo this breaks tests

    let current = response.data.catalog.search_store.elements.iter()
        .filter(|game|{
            match &game.promotions {
                None => false,
                Some(promotions) => {
                    promotions.promotional_offers.iter()
                        .flat_map(|offers|&offers.promotional_offers)
                        .any(|promotion| {
                            promotion.start_date < now
                                && now < promotion.end_date
                                && promotion.discount_setting.discount_percentage == 0
                        })
                }
            }
        })
        .collect::<Vec<_>>();

    current.into_iter()
        .map(|element|Game{ title: element.title.clone(), platform: Platform::Epic })
        .collect::<Vec<_>>()
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    data: Data,
}

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    #[serde(rename = "Catalog")]
    catalog: Catalog,
}

#[derive(Serialize, Deserialize, Debug)]
struct Catalog {
    #[serde(rename = "searchStore")]
    search_store: SearchStore,
}

#[derive(Serialize, Deserialize, Debug)]
struct SearchStore {
    elements: Vec<Element>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Element {
    title: String,
    #[serde(rename = "effectiveDate")]
    effective_date: DateTime<Utc>,
    promotions: Option<Promotions>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Promotions {
    #[serde(rename = "promotionalOffers")]
    promotional_offers: Vec<PromotionalOffers>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PromotionalOffers {
    #[serde(rename = "promotionalOffers")]
    promotional_offers: Vec<PromotionalOffer>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PromotionalOffer {
    #[serde(rename = "startDate")]
    start_date: DateTime<Utc>,
    #[serde(rename = "endDate")]
    end_date: DateTime<Utc>,
    #[serde(rename = "discountSetting")]
    discount_setting: DiscountSetting,
}

#[derive(Serialize, Deserialize, Debug)]
struct DiscountSetting {
    #[serde(rename = "discountPercentage")]
    discount_percentage: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Platform;

    #[test]
    fn parse_and_filter_epic() {
        let result = parse_and_filter(include_str!("epic_response.json"));

        assert_eq!(result, vec![
           Game {
               title: "Sheltered".to_string(),
               platform: Platform::Epic
           },
           Game {
               title: "Nioh: The Complete Edition".to_string(),
               platform: Platform::Epic
           },
        ]);
    }
}
