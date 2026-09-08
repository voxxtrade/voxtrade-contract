from stellar_sdk import Keypair, Server, TransactionBuilder, Network

class VoiceAgentSigner:
    def __init__(self, secret_key: str):
        self.keypair = Keypair.from_secret(secret_key)
        self.server = Server("https://soroban-testnet.stellar.org:443")

    def sign_x402_lock(self, treasury_contract: str, amount: int):
        # Programmatically signs the cross-contract invocation
        pass

