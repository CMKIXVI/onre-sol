#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct RedemptionLedger {
    requested: u128,
    vault_token_in: u128,
    settled_token_in: u128,
    returned_token_in: u128,
}

impl RedemptionLedger {
    fn create(&mut self, amount: u128) {
        self.requested += amount;
        self.vault_token_in += amount;
    }

    fn fulfill(&mut self, amount: u128) {
        assert!(amount <= self.requested);
        assert!(amount <= self.vault_token_in);
        self.requested -= amount;
        self.vault_token_in -= amount;
        self.settled_token_in += amount;
    }

    fn cancel(&mut self, amount: u128) {
        assert!(amount <= self.requested);
        assert!(amount <= self.vault_token_in);
        self.requested -= amount;
        self.vault_token_in -= amount;
        self.returned_token_in += amount;
    }

    fn assert_conserved(&self, created: u128) {
        assert_eq!(
            created,
            self.vault_token_in + self.settled_token_in + self.returned_token_in,
            "created redemption tokens must reconcile to vault, settled, and returned buckets"
        );
        assert_eq!(
            self.requested,
            self.vault_token_in,
            "pending redemption liability must equal tokens still locked in the vault"
        );
    }
}

#[test]
fn create_partial_fulfill_cancel_reconciles() {
    let mut ledger = RedemptionLedger::default();
    let first_request = 1_000u128;

    ledger.create(first_request);
    ledger.assert_conserved(first_request);

    ledger.fulfill(400);
    ledger.assert_conserved(first_request);
    assert_eq!(ledger.requested, 600);
    assert_eq!(ledger.settled_token_in, 400);

    ledger.cancel(600);
    ledger.assert_conserved(first_request);
    assert_eq!(ledger.requested, 0);
    assert_eq!(ledger.vault_token_in, 0);
    assert_eq!(ledger.returned_token_in, 600);
}

#[test]
fn multiple_requests_reconcile_independently_and_globally() {
    let mut ledger = RedemptionLedger::default();
    let first = 1_000u128;
    let second = 750u128;

    ledger.create(first);
    ledger.create(second);
    ledger.assert_conserved(first + second);

    ledger.fulfill(250);
    ledger.assert_conserved(first + second);

    ledger.cancel(500);
    ledger.assert_conserved(first + second);

    ledger.fulfill(600);
    ledger.assert_conserved(first + second);

    assert_eq!(ledger.requested, 400);
    assert_eq!(ledger.vault_token_in, 400);
    assert_eq!(ledger.settled_token_in, 850);
    assert_eq!(ledger.returned_token_in, 500);
}

#[test]
fn cancellation_returns_only_the_remaining_request() {
    let mut ledger = RedemptionLedger::default();
    let original = 900u128;

    ledger.create(original);
    ledger.fulfill(350);

    // The remaining cancellable amount is original - fulfilled, not original.
    let remaining = original - 350;
    ledger.cancel(remaining);
    ledger.assert_conserved(original);

    assert_eq!(ledger.returned_token_in, remaining);
    assert_eq!(ledger.settled_token_in, 350);
    assert_eq!(ledger.requested, 0);
    assert_eq!(ledger.vault_token_in, 0);
}
