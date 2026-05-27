#[cfg(test)] ///////
mod tests {
    use soroban_sdk::{testutils::Address as _, Address, Env, String};
    use soroban_sdk::token;

    use crate::{AuctionContract, AuctionContractClient};

    fn setup_token(env: &Env) -> (Address, token::StellarAssetClient<'static>, token::Client<'static>) {
        let admin = Address::generate(env);
        let contract = env.register_stellar_asset_contract(admin.clone());
        (contract.clone(), token::StellarAssetClient::new(env, &contract), token::Client::new(env, &contract))
    }

    #[test]
    fn stores_auctions_in_contract_storage() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(AuctionContract, ());
        let client = AuctionContractClient::new(&env, &contract_id);

        let (asset, token_admin, token) = setup_token(&env);
        let treasury = Address::generate(&env);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        token_admin.mint(&alice, &1000);
        token_admin.mint(&bob, &1000);

        let name = String::from_str(&env, "vip.xlm");

        client.create_auction(&name, &asset, &treasury, &200, &10, &20);
        client.place_bid(&name, &alice, &500, &12);
        client.place_bid(&name, &bob, &300, &13);

        let settlement = client.settle(&name, &21).unwrap();
        assert_eq!(settlement.winner, Some(alice));
        assert_eq!(settlement.clearing_price, 300);
        assert!(settlement.sold);

        assert_eq!(token.balance(&alice), 1000 - 300);
        assert_eq!(token.balance(&bob), 1000);
        assert_eq!(token.balance(&treasury), 300);
    } //

    #[test]
    fn test_auction_no_bids() {
        let env = Env::default();
        let contract_id = env.register(AuctionContract, ());
        let client = AuctionContractClient::new(&env, &contract_id);

        let (asset, _, _) = setup_token(&env);
        let treasury = Address::generate(&env);
        let name = String::from_str(&env, "ghost.xlm");
        client.create_auction(&name, &asset, &treasury, &100, &10, &20);

        let settlement = client.settle(&name, &21);
        assert!(settlement.is_none());
    }

    #[test]
    fn test_auction_reserve_not_met() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(AuctionContract, ());
        let client = AuctionContractClient::new(&env, &contract_id);

        let (asset, token_admin, token) = setup_token(&env);
        let treasury = Address::generate(&env);
        let alice = Address::generate(&env);

        token_admin.mint(&alice, &1000);
        let name = String::from_str(&env, "cheap.xlm");
        client.create_auction(&name, &asset, &treasury, &1000, &10, &20);
        client.place_bid(&name, &alice, &500, &15);

        let settlement = client.settle(&name, &21).unwrap();
        assert_eq!(settlement.winner, None);
        assert_eq!(settlement.clearing_price, 0);
        assert!(!settlement.sold);

        assert_eq!(token.balance(&alice), 1000);
        assert_eq!(token.balance(&treasury), 0);
    }

    #[test]
    fn test_auction_tie_behavior() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(AuctionContract, ());
        let client = AuctionContractClient::new(&env, &contract_id);

        let (asset, token_admin, token) = setup_token(&env);
        let treasury = Address::generate(&env);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        token_admin.mint(&alice, &1000);
        token_admin.mint(&bob, &1000);
        let name = String::from_str(&env, "tie.xlm");
        client.create_auction(&name, &asset, &treasury, &100, &10, &20);

        client.place_bid(&name, &alice, &500, &12);
        client.place_bid(&name, &bob, &500, &13);

        let settlement = client.settle(&name, &21).unwrap();
        // First bidder wins in case of tie in current implementation
        assert_eq!(settlement.winner, Some(alice));
        assert_eq!(settlement.clearing_price, 500);
        assert!(settlement.sold);

        assert_eq!(token.balance(&alice), 1000 - 500);
        assert_eq!(token.balance(&bob), 1000);
        assert_eq!(token.balance(&treasury), 500);
    }

    #[test]
    fn test_auction_clearing_price_logic() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(AuctionContract, ());
        let client = AuctionContractClient::new(&env, &contract_id);

        let (asset, token_admin, token) = setup_token(&env);
        let treasury = Address::generate(&env);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let charlie = Address::generate(&env);

        token_admin.mint(&alice, &1000);
        token_admin.mint(&bob, &1000);
        token_admin.mint(&charlie, &1000);
        let name = String::from_str(&env, "multi.xlm");
        client.create_auction(&name, &asset, &treasury, &100, &10, &20);

        client.place_bid(&name, &alice, &1000, &12);
        client.place_bid(&name, &bob, &500, &13);
        client.place_bid(&name, &charlie, &750, &14);

        let settlement = client.settle(&name, &21).unwrap();
        assert_eq!(settlement.winner, Some(alice));
        assert_eq!(settlement.clearing_price, 750); // Second highest bid
        assert!(settlement.sold);

        assert_eq!(token.balance(&alice), 1000 - 750);
        assert_eq!(token.balance(&bob), 1000);
        assert_eq!(token.balance(&charlie), 1000);
        assert_eq!(token.balance(&treasury), 750);
    }
}
