"""Handling of prepared ledger requests."""

from typing import Union

from . import bindings
from .error import VdrError, VdrErrorCode


class Request:
    """A prepared ledger request."""

    def __init__(self, handle: bindings.RequestHandle):
        """Initialize the `Request` instance."""
        self.handle = handle

    @property
    def body(self) -> str:
        """Get the body of the prepared request as a JSON string."""
        if not self.handle:
            raise VdrError(VdrErrorCode.WRAPPER, "no request handle")
        return bindings.request_get_body(self.handle)

    def free(self):
        """Release the prepared request."""
        if hasattr(self, "handle") and self.handle:
            bindings.request_free(self.handle)
            self.handle = None

    @property
    def signature_input(self) -> bytes:
        """Get the signature input of this prepared request.

        This value should be passed to the signer in order to generate an
        appropriate signature for the request.

        Returns:
            A bytes instance with the canonical representation of the request.
        """
        if not self.handle:
            raise VdrError(VdrErrorCode.WRAPPER, "no request handle")
        return bindings.request_get_signature_input(self.handle)

    def set_endorser(self, endorser: str):
        """
        Set the endorser property of an existing request.

        When a transaction is expected to be sent to the ledger by an endorser, this
        property must be set in order to generate a valid signature.
        Note: Both Transaction Author and Endorser must sign the output request.
        More about Transaction Endorser:
            https://github.com/hyperledger/indy-node/blob/master/design/transaction_endorser.md
            https://github.com/hyperledger/indy-sdk/blob/master/docs/configuration.md

        Args:
            endorser_did: DID of the Endorser that will submit the transaction.
                The Endorser's DID must be present on the ledger.
        """
        if not self.handle:
            raise VdrError(VdrErrorCode.WRAPPER, "no request handle")
        bindings.request_set_endorser(self.handle, endorser)

    def set_signature(self, signature: bytes):
        """Set the signature on a prepared request.

        Args:
            signature: A bytes instance with the ed25519 signature
                generated over `request.signature_input`
        """
        if not self.handle:
            raise VdrError(VdrErrorCode.WRAPPER, "no request handle")
        bindings.request_set_signature(self.handle, signature)

    def set_txn_author_agreement_acceptance(self, acceptance: Union[str, dict]):
        """Set the TAA acceptance on a prepared request.

        Args:
            acceptance: Normally be the output of
                `ledger.prepare_txn_agreement_acceptance`, which may be
                cached for the current session
        """
        if not self.handle:
            raise VdrError(VdrErrorCode.WRAPPER, "no request handle")
        bindings.request_set_txn_author_agreement_acceptance(self.handle, acceptance)

    def __del__(self):
        """Release the pool instance."""
        self.free()

    def __repr__(self) -> str:
        """Format the pool instance as a debug string."""
        if self.handle:
            status = self.handle
        else:
            status = "freed"
        return f"{self.__class__.__name__}({status})"
