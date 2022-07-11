using System;
using System.Threading.Tasks;
using static indy_vdr_dotnet.models.Structures;

namespace indy_vdr_dotnet.libindy_vdr
{
    public static class LedgerApi
    {
        /// <summary>
        /// Builds a <c>SET_TXN_AUTHR_AGRMT_AML</c> request.
        /// </summary>
        /// <remarks> 
        /// Request to add a new list of acceptance mechanisms for transaction author
        /// agreement. Acceptance Mechanism is a description of the ways how the user may
        /// accept a transaction author agreement.
        /// </remarks>
        /// <param name="submitterDid">Identifier (DID) of the transaction author as a base58-encoded string.</param>
        /// <param name="aml">A set of new acceptance mechanisms:
        ///    {
        ///        "acceptance mechanism label 1": { description 1},
        ///        "acceptance mechanism label 2": { description 2},
        ///        ...
        ///    }
        /// </param>
        /// <param name="verion">The version of the new acceptance mechanisms. (Note: unique on the Ledger)</param>
        /// <param name="amlContext">(Optional) common context information about acceptance mechanisms (may be a URL to external resource).</param>
        /// <exception cref="IndyVdrException">Throws if any parameter is invalid.</exception>
        /// <returns>Returns the <see cref="IntPtr"/> handle to a <c>SET_TXN_AUTHR_AGRMT_AML</c> request.</returns>
        public static async Task<IntPtr> BuildAcceptanceMechanismsRequestAsync(
            string submitterDid,
            string aml,
            string verion,
            string amlContext = null)
        {
            IntPtr requestHandle = new();
            int errorCode = NativeMethods.indy_vdr_build_acceptance_mechanisms_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(aml),
                FfiStr.Create(verion),
                FfiStr.Create(amlContext),
                ref requestHandle);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }

            return requestHandle;
        }

        /// <summary>
        /// Builds a <c>GET_TXN_AUTHR_AGRMT_AML</c> request.
        /// </summary>
        /// <remarks>
        /// Request to get a list of acceptance mechanisms from the ledger valid for specified time, or the latest one.
        /// </remarks>
        /// <param name="timestamp">(Optional) Unix timestamp to get an active acceptance mechanisms. The latest one will be returned for the empty timestamp.</param>
        /// <param name="version">(Optional) version of acceptance mechanisms.</param>
        /// <param name="submitterDid">(Optional) DID of the read request sender (if not provided, then the default Libindy DID will be used).</param>
        /// <exception cref="IndyVdrException">Throws if any parameter is invalid.</exception>
        /// <returns>Returns the <see cref="IntPtr"/> handle to a <c>GET_TXN_AUTHR_AGRMT_AML</c> request.</returns>
        public static async Task<IntPtr> BuildGetAcceptanceMechanismsRequestAsync(
            long timestamp = 0,
            string version = null,
            string submitterDid = null)
        {
            IntPtr requestHandle = new();
            int errorCode = NativeMethods.indy_vdr_build_get_acceptance_mechanisms_request(
                FfiStr.Create(submitterDid),
                timestamp,
                FfiStr.Create(version),
                ref requestHandle);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }
            return requestHandle;
        }

        /// <summary>
        /// Builds an <c>ATTRIB</c> request.
        /// </summary>
        /// <remarks>
        /// Request to add attribute to a NYM record.
        /// </remarks>
        /// <param name="targetDid">Target DID as base58-encoded string for 16 or 32 bit DID value.</param>
        /// <param name="submitterDid">(Optional) Identifier (DID) of the transaction author as base58-encoded string.</param>
        /// <param name="hash">(Optional) Hash of attribute data.</param>
        /// <param name="raw">(Optional) JSON, where key is attribute name and value is attribute value.</param>
        /// <param name="enc">(Optional) Encrypted value attribute data.</param>
        /// <exception cref="IndyVdrException">Throws if any parameter is invalid.</exception>
        /// <returns>Returns the <see cref="IntPtr"/> handle to a <c>ATTRIB</c> request.</returns>
        public static async Task<IntPtr> BuildAttributeRequest(
            string targetDid,
            string submitterDid = null,
            string hash = null,
            string raw = null,
            string enc = null)
        {
            IntPtr requestHandle = new();
            int errorCode = NativeMethods.indy_vdr_build_attrib_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(targetDid),
                FfiStr.Create(hash),
                FfiStr.Create(raw),
                FfiStr.Create(enc),
                ref requestHandle);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }

            return requestHandle;
        }
        /// <summary>
        /// Builds a <c>GET_ATTRIB</c> request.
        /// </summary>
        /// <remarks>
        /// Request to get information about an Attribute for the specified DID.
        /// </remarks>
        /// <param name="targetDid">Target DID as base58-encoded string for 16 or 32 bit DID value.</param>
        /// <param name="submitterDid">(Optional) DID of the read request sender (if not provided, then the default Libindy DID will be used).</param>
        /// <param name="hash">(Optional) Requested attribute name.</param>
        /// <param name="raw">(Optional) Requested attribute hash.</param>
        /// <param name="enc">(Optional) Requested attribute encrypted value.</param>
        /// <exception cref="IndyVdrException">Throws if any parameter is invalid.</exception>
        /// <returns>Returns the <see cref="IntPtr"/> handle to a <c>GET_ATTRIB</c> request.</returns>
        public static async Task<IntPtr> BuildGetAttributeRequest(
            string targetDid,
            string submitterDid = null,
            string hash = null,
            string raw = null,
            string enc = null)
        {
            IntPtr requestHandle = new();
            int errorCode = NativeMethods.indy_vdr_build_get_attrib_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(targetDid),
                FfiStr.Create(hash),
                FfiStr.Create(raw),
                FfiStr.Create(enc),
                ref requestHandle);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }

            return requestHandle;
        }

        /// <summary>
        /// Builds a <c>CRED_DEF</c> request to to add a credential definition to the ledger.
        /// </summary>
        /// <remarks>
        /// In particular, this publishes the public key that the issuer created for
        /// issuing credentials against a particular schema.</remarks>
        /// <param name="submitterDid">Identifier (DID) of the transaction author as base58-encoded string</param>
        /// <param name="credDef">Credential definition.
        /// <c>
        /// {
        ///     "id": "credential definition identifier",
        ///     "schemaId": "schema identifier",
        ///     "type": "CL",
        ///         // type of the credential definition. CL is currently
        ///         // the only supported type
        ///     "tag": "",
        ///         // allows to distinguish between credential definitions
        ///         // for the same issuer and schema
        ///     "value": /* Dictionary with Credential Definition's data: */ 
        ///             {
        ///                 "primary": "primary credential public key",
        ///                 "revocation": /* Optional */ "revocation credential public key"
        ///             },
        ///     "ver": Version of the CredDef json>
        /// }
        /// </c>
        ///</param>
        ///<exception cref="IndyVdrException">Throws if any parameter is invalid.</exception>
        ///<returns>Returns the <see cref="IntPtr"/>handle to a <c>CRED_DEF</c> request.</returns>
        public static async Task<IntPtr> BuildCredDefRequest(
            string submitterDid,
            string credDef)
        {
            IntPtr requestHandle = new();
            int errorCode = NativeMethods.indy_vdr_build_cred_def_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(credDef),
                ref requestHandle);


            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }

            return requestHandle;
        }

        /// <summary>
        /// Builds a custom request.
        /// </summary>
        /// <param name="requestJson">Json <see cref="System.String"/> of a custom request.</param>
        /// <exception cref="IndyVdrException">Throws if <paramref name="requestJson"/> is invalid.</exception>
        /// <returns>Returns the <see cref="IntPtr"/> handle to a custom request.</returns>
        public static async Task<IntPtr> BuildCustomRequest(
            string requestJson)
        {
            IntPtr requestHandle = new();
            int errorCode = NativeMethods.indy_vdr_build_custom_request(
                FfiStr.Create(requestJson),
                ref requestHandle);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }
            return requestHandle;
        }

        /// <summary>
        /// Builds a <c>DISABLE_ALL_TXN_AUTHR_AGRMTS</c> request.
        /// </summary>
        /// <remarks>
        /// Used to disable all Transaction Author Agreements on the ledger.
        /// </remarks>
        /// <param name="submitterDid">Identifier (DID) of the transaction author as base58-encoded string.</param>
        /// <exception cref="IndyVdrException">Throws if <paramref name="submitterDid"/> is invalid.</exception>
        /// <returns>Returns the <see cref="IntPtr"/> handle to a <c>DISABLE_ALL_TXN_AUTHR_AGRMTS</c> request.</returns>
        public static async Task<IntPtr> BuildDisableAllTxnAuthorAgreementsRequest(
            string submitterDid)
        {
            IntPtr requestHandle = new();
            int errorCode = NativeMethods.indy_vdr_build_disable_all_txn_author_agreements_request(
                FfiStr.Create(submitterDid),
                ref requestHandle);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }
            return requestHandle;
        }

        /// <summary>
        /// Builds a <c>GET_CRED_DEF</c> request to fetch a credential definition by ID.
        /// </summary>
        /// <param name="credDefDid">ID of the corresponding credential definition on the ledger.</param>
        /// <param name="submitterDid">(Optional) DID of the read request sender. If not provided then the default Libindy DID will be used.</param>
        /// <exception cref="IndyVdrException">Throws if any parameter is invalid.</exception>
        /// <returns>Returns the <see cref="IntPtr"/> handle to a <c>GET_CRED_DEF</c> request.</returns>
        public static async Task<IntPtr> BuildGetCredDefRequest(
            string credDefDid,
            string submitterDid = null)
        {
            IntPtr requestHandle = new();
            int errorCode = NativeMethods.indy_vdr_build_get_cred_def_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(credDefDid),
                ref requestHandle);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }
            return requestHandle;
        }

        /// <summary>
        /// Builds a   request to get information about a DID (NYM).
        /// </summary>
        /// <param name="targetDid">Target DID as base58-encoded string for 16 or 32 bit DID value.</param>
        /// <param name="submitterDid">(Optional) DID of the read request sender. If not provided then the default Libindy DID will be use).</param>
        /// <exception cref="IndyVdrException">Throws if any parameter is invalid.</exception>
        /// <returns>Returns the <see cref="IntPtr"/> handle to a <c>GET_NYM</c> request.</returns>
        public static async Task<IntPtr> BuildGetNymRequest(
            string targetDid,
            string submitterDid = null)
        {
            IntPtr requestHandle = new();
            int errorCode = NativeMethods.indy_vdr_build_get_nym_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(targetDid),
                ref requestHandle);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }
            return requestHandle;
        }

        /// <summary>
        /// Builds a <c>GET_REVOC_REG_DEF</c> request.
        /// </summary>
        /// <remarks>
        /// Request to get the revocation registry definition for a given revocation registry ID.
        /// </remarks>
        /// <param name="revocRegId">ID of the corresponding revocation registry definition.</param>
        /// <param name="submitterDid">(Optional) DID of the read request sender. If not provided then the default Libindy DID will be used.</param>
        /// <exception cref="IndyVdrException">Throws if any parameter is invalid.</exception>
        /// <returns>Returns the <see cref="IntPtr"/> handle to a <c>GET_REVOC_REG_DEF</c> request.</returns>
        public static async Task<IntPtr> BuildGetRevocRegDefRequest(
            string revocRegId,
            string submitterDid = null)
        {
            IntPtr requestHandle = new();
            int errorCode = NativeMethods.indy_vdr_build_get_revoc_reg_def_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(revocRegId),
                ref requestHandle);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }
            return requestHandle;
        }

        /// <summary>
        /// Builds a <c>GET_REVOC_REG</c> request.
        /// </summary>
        /// <remarks>
        /// Request to get the accumulated state of the revocation registry by ID. The state is defined by the given timestamp.
        /// </remarks>
        /// <param name="revocRegId">ID of the corresponding revocation registry definition.</param>
        /// <param name="timestamp">Requested time represented as a total number of seconds since the Unix epoch.</param>
        /// <param name="submitterDid">(Optional) DID of the read request sender. If not provided then the default Libindy DID will be used.</param>
        /// <exception cref="IndyVdrException">Throws if any parameter is invalid.</exception>
        /// <returns>Returns the <see cref="IntPtr"/> handle to a <c>GET_REVOC_REG</c> request.</returns>
        public static async Task<IntPtr> BuildGetRevocRegRequest(
            string revocRegId,
            long timestamp,
            string submitterDid = null)
        {
            IntPtr requestHandle = new();
            int errorCode = NativeMethods.indy_vdr_build_get_revoc_reg_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(revocRegId),
                timestamp,
                ref requestHandle);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }
            return requestHandle;
        }

        /// <summary>
        /// Builds a <c>GET_REVOC_REG_DELTA</c> request.
        /// </summary>
        /// <remarks>
        /// Request to get the delta of the accumulated state of the revocation registry
        /// identified by `<c>revoc_reg_id</c>`. The delta is defined by from and to timestamp fields.
        /// If from is not specified, then the whole state until `<c>to</c>` will be returned.
        /// </remarks>
        /// <param name="revocRegId">ID of the corresponding revocation registry definition.</param>
        /// <param name="toTs">Requested time represented as a total number of seconds from Unix epoch.</param>
        /// <param name="fromTs">Requested time represented as a total number of seconds from Unix epoch.</param>
        /// <param name="submitterDid">(Optional) DID of the read request sender. If not provided then the default Libindy DID will be used.</param>
        /// <exception cref="IndyVdrException">Throws if any parameter is invalid.</exception>
        /// <returns>Returns the <see cref="IntPtr"/> handle to a <c>GET_REVOC_REG_DELTA</c> request.</returns>
        public static async Task<IntPtr> BuildGetRevocRegDeltaRequestAsync(
            string revocRegId,
            long toTs,
            long fromTs = -1,
            string submitterDid = null)
        {
            IntPtr requestHandle = new();
            int errorCode = NativeMethods.indy_vdr_build_get_revoc_reg_delta_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(revocRegId),
                fromTs,
                toTs,
                ref requestHandle);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }

            return requestHandle;
        }

        /// <summary>
        /// Builds a <c>GET_SCHEMA</c> request to fetch a credential schema by ID.
        /// </summary>
        /// <param name="schemaId">ID of the corresponding schema on the ledger.</param>
        /// <param name="submitterDid">(Optional) DID of the read request sender. If not provided then the default Libindy DID will be used.</param>
        /// <exception cref="IndyVdrException">Throws if any parameter is invalid.</exception>
        /// <returns>Returns the <see cref="IntPtr"/> handle to a <c>GET_SCHEMA</c> request.</returns>
        public static async Task<IntPtr> BuildGetSchemaRequestAsync(
            string schemaId,
            string submitterDid = null)
        {
            IntPtr requestHandle = new();
            int errorCode = NativeMethods.indy_vdr_build_get_schema_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(schemaId),
                ref requestHandle);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }

            return requestHandle;
        }

        /// <summary>
        /// Builds a <c>GET_TXN_AUTHR_AGRMT</c> request.
        /// </summary>
        /// <remarks>
        /// Used to get a specific Transaction Author Agreement from the ledger.
        /// </remarks>
        /// <param name="submitterDid">(Optional) DID of the read request sender. If not provided then the default Libindy DID will be used.</param>
        /// <param name="data">(Optional) specifies conditions for getting a specific TAA. Contains 3 mutually exclusive optional fields:
        /// {
        ///     hash: Optional<str> - hash of requested TAA,
        ///     version: Optional<str> - version of requested TAA.
        ///     imestamp: Optional<i64> - ledger will return TAA valid at requested timestamp.
        /// }
        /// Null data or empty JSON are acceptable here. In this case, ledger will return the latest version of the TAA.
        /// </param>
        /// <exception cref="IndyVdrException">Throws if any parameter is invalid.</exception>
        /// <returns>Returns the <see cref="IntPtr"/> handle to a <c>GET_TXN_AUTHR_AGRMT</c> request.</returns>
        public static async Task<IntPtr> BuildGetTxnAuthorAgreementRequestAsync(
            string submitterDid = null,
            string data = null)
        {
            IntPtr requestHandle = new();
            int errorCode = NativeMethods.indy_vdr_build_get_txn_author_agreement_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(data),
                ref requestHandle);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }

            return requestHandle;
        }

        /// <summary>
        /// Builds a <c>GET_TXN request</c> to get any transaction by its sequence number.
        /// </summary>
        /// <param name="ledgerType">Type of the ledger the requested transaction belongs to pass a `<c>LedgerType</c>` instance for known values.</param>
        /// <param name="seqNo">Requested transaction sequence number as it's stored on the ledger.</param>
        /// <param name="submitterDid">(Optional) DID of the read request sender. If not provided then the default Libindy DID will be used.</param>
        /// <exception cref="IndyVdrException">Throws if any parameter is invalid.</exception>
        /// <returns>Returns the <see cref="IntPtr"/> handle to a <c>GET_TXN request</c> request.</returns>
        public static async Task<IntPtr> BuildGetTxnRequestAsync(
            int ledgerType,
            int seqNo,
            string submitterDid = null)
        {
            IntPtr requestHandle = new();
            int errorCode = NativeMethods.indy_vdr_build_get_txn_request(
                FfiStr.Create(submitterDid),
                ledgerType,
                seqNo,
                ref requestHandle);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }

            return requestHandle;
        }
        /// <summary>
        /// Builds a <c>GET_VALIDATOR_INFO</c> request.
        /// </summary>
        /// <param name="submitterDid">DID of the request sender.</param>
        /// <exception cref="IndyVdrException">Throws if <paramref name="submitterDid"/> is invalid.</exception>
        /// <returns>Returns the <see cref="IntPtr"/> handle to a <c>GET_VALIDATOR_INFO</c> request.</returns>
        public static async Task<IntPtr> BuildGetValidatorInfoRequestAsync(
            string submitterDid)
        {
            IntPtr requestHandle = new();
            int errorCode = NativeMethods.indy_vdr_build_get_validator_info_request(
                FfiStr.Create(submitterDid),
                ref requestHandle);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }

            return requestHandle;
        }
        /// <summary>
        /// Builds a <c>NYM</c> request to create new DID on the ledger.
        /// </summary>
        /// <param name="submitterDid">Identifier (DID) of the transaction author as base58-encoded string.</param>
        /// <param name="dest">Target DID as base58-encoded string for 16 or 32 bit DID value.</param>
        /// <param name="verkey">(Optional) Target identity verification key as base58-encoded string.</param>
        /// <param name="alias">(Optional) The NYM's alias.</param>
        /// <param name="role">(Optional) Role of a user NYM record:
        ///    <c>null</c> (common USER),
        ///    <c>TRUSTEE</c>,
        ///    <c>STEWARD</c>,
        ///    <c>TRUST_ANCHOR</c>,
        ///    <c>ENDORSER</c> - equal to TRUST_ANCHOR that will be removed soon,
        ///    <c>NETWORK_MONITOR</c>,
        ///    empty string to reset role</param>
        /// <exception cref="IndyVdrException">Throws if any parameter is invalid.</exception>
        /// <returns>Returns the <see cref="IntPtr"/> handle to a <c>NYM</c> request.</returns>
        public static async Task<IntPtr> BuildNymRequestAsync(
            string submitterDid,
            string dest,
            string verkey = null,
            string alias = null,
            string role = null)
        {
            IntPtr requestHandle = new();
            int errorCode = NativeMethods.indy_vdr_build_nym_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(dest),
                FfiStr.Create(verkey),
                FfiStr.Create(alias),
                FfiStr.Create(role),
                ref requestHandle);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }

            return requestHandle;
        }

        /// <summary>
        /// Builds a <c>REVOC_REG_DEF</c> request.
        /// </summary>
        /// <remarks>
        /// Request to add the definition of revocation registry to an existing credential definition.
        /// </remarks>
        /// <param name="submitterDid">Request to add the definition of revocation registry to an existing credential definition.</param>
        /// <param name="revocRegDefJson">revoc_reg_def: Revocation Registry data:
        /// jsonc
        /// {
        ///     "id": "revocation registry identifier",
        ///     "revocDefType": "CL_ACCUM",
        ///         // revocation registry type (only CL_ACCUM is supported for now)
        ///     "tag": "", // Unique descriptive ID of the registry definition
        ///     "credDefId": "credential definition ID",
        ///     "value": /* Registry-specific data */ {
        ///         "issuanceType": "ISSUANCE_BY_DEFAULT",
        ///          // Type of issuance: ISSUANCE_BY_DEFAULT or ISSUANCE_ON_DEMAND
        ///      "maxCredNum": 10000,
        ///          // Maximum number of credentials the Registry can serve.
        ///      "tailsHash": "sha256 hash of tails file in base58",
        ///      "tailsLocation": "URL or path for the tails file",
        ///      "publicKeys": { /* <public_keys> */ } // registry's public keys
        ///  },
        ///  "ver": "version of revocation registry definition json"
        ///}
        ///</param>
        /// <exception cref="IndyVdrException">Throws if any parameter is invalid.</exception>
        /// <returns>Returns the <see cref="IntPtr"/> handle to a <c>REVOC_REG_DEF</c> request.</returns>
        public static async Task<IntPtr> BuildRevocRegDefRequestAsync(
            string submitterDid,
            string revocRegDefJson)
        {
            IntPtr requestHandle = new();
            int errorCode = NativeMethods.indy_vdr_build_revoc_reg_def_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(revocRegDefJson),
                ref requestHandle);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }

            return requestHandle;
        }
        /// <summary>
        /// Builds a <c>REVOC_REG_ENTRY</c> request.
        /// </summary>
        /// <remarks>
        /// Request to add the revocation registry entry containing the new accumulator value and issued/revoked indices. 
        /// This is just a delta of indices, not the whole list. 
        /// It can be sent each time a new credential is issued/revoked.
        /// </remarks>
        /// <param name="submitterDid">Identifier (DID) of the transaction author as base58-encoded string.</param>
        /// <param name="revocRegDefId"></param>
        /// <param name="revocRegDefType"></param>
        /// <param name="revocRegEntryJson"> Registry-specific data:
        /// {
        ///     "value": {
        ///         "prevAccum": "previous accumulator value",
        ///         "accum": "current accumulator value",
        ///         "issued": [], // array<number> - an array of issued indices
        ///         "revoked": [] // array<number> an array of revoked indices
        ///              },
        ///     "ver": "version of the revocation registry entry json"
        /// }
        /// </param>
        /// <exception cref="IndyVdrException">Throws if any parameter is invalid.</exception>
        /// <returns>Returns the <see cref="IntPtr"/> handle to a <c>REVOC_REG_ENTRY</c> request.</returns>
        public static async Task<IntPtr> BuildRevocRegEntryRequestAsync(
            string submitterDid,
            string revocRegDefId,
            string revocRegDefType,
            string revocRegEntryJson)
        {
            IntPtr requestHandle = new();
            int errorCode = NativeMethods.indy_vdr_build_revoc_reg_entry_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(revocRegDefId),
                FfiStr.Create(revocRegDefType),
                FfiStr.Create(revocRegEntryJson),
                ref requestHandle);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }

            return requestHandle;
        }

        /// <summary>
        /// Builds a <c>SCHEMA</c> request to to add a credential schema to the ledger.
        /// </summary>
        /// <param name="submitterDid">Identifier (DID) of the transaction author as base58-encoded string</param>
        /// <param name="schemaJson">Credential schema:
        /// {
        ///     "id": "identifier of schema",
        ///     "attrNames": "array of attribute name strings (the number of attributes should be less or equal than 125)",
        ///     "name": "schema's name string",
        ///     "version": "schema's version string",
        ///     "ver": "version of the schema json"
        ///}
        /// </param>
        /// <exception cref="IndyVdrException">Throws if any parameter is invalid.</exception>
        /// <returns>Returns the <see cref="IntPtr"/> handle to a <c>SCHEMA</c> request.</returns>
        public static async Task<IntPtr> BuildSchemaRequestAsync(
            string submitterDid,
            string schemaJson)
        {
            IntPtr requestHandle = new();
            int errorCode = NativeMethods.indy_vdr_build_schema_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(schemaJson),
                ref requestHandle);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }
            return requestHandle;
        }

        /// <summary>
        /// Builds a <c>TXN_AUTHR_AGRMT</c> request.
        /// </summary>
        /// <remarks>
        /// Used to add a new version of the Transaction Author Agreement to the ledger.
        /// </remarks>
        /// <param name="submitterDid">Identifier (DID) of the transaction author as base58-encoded string.</param>
        /// <param name="text">(Optional) the content of the TAA. Mandatory in case of adding a new TAA.
        /// An existing TAA text can not be changed.
        /// For Indy Node version <= 1.12.0:
        ///     Use empty string to reset TAA on the ledger
        /// For Indy Node version > 1.12.0:
        ///     Should be omitted in case of updating an existing TAA (setting `retirement_ts`)</param>
        /// <param name="version">The version of the TAA (a unique UTF-8 string).</param>
        /// <param name="ratificationTs">(Optional) the date (timestamp) of TAA ratification by network government.
        /// For Indy Node version <= 1.12.0:
        ///     Must be omitted
        /// For Indy Node version > 1.12.0:
        ///     Must be specified in case of adding a new TAA
        ///     Can be omitted in case of updating an existing TAA</param>
        /// <param name="retirementTs">(Optional) the date (timestamp) of TAA retirement.</param>
        /// <exception cref="IndyVdrException">Throws if any parameter is invalid.</exception>
        /// <returns>Returns the <see cref="IntPtr"/> handle to a <c>TXN_AUTHR_AGRMT</c> request.</returns>
        public static async Task<IntPtr> BuildTxnAuthorAgreementRequestAsync(
            string submitterDid,
            string text,
            string version,
            long ratificationTs,
            long retirementTs)
        {
            IntPtr requestHandle = new();
            int errorCode = NativeMethods.indy_vdr_build_txn_author_agreement_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(text),
                FfiStr.Create(version),
                ratificationTs,
                retirementTs,
                ref requestHandle);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }

            return requestHandle;
        }
    }
}
