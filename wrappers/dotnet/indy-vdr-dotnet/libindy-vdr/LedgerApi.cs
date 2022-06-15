using System;
using System.Diagnostics;
using System.Threading.Tasks;
using static indy_vdr_dotnet.models.Structures;

namespace indy_vdr_dotnet.libindy_vdr
{
    public static class LedgerApi
    {
        public static async Task<uint> BuildAcceptanceMechanismsRequestAsync(
            string submitterDid,
            string aml,
            string verion,
            string amlContext = null)
        {
            uint requestHandle = 0;
            int errorCode = NativeMethods.indy_vdr_build_acceptance_mechanisms_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(aml),
                FfiStr.Create(verion),
                FfiStr.Create(amlContext),
                ref requestHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }

            return requestHandle;
        }

        public static async Task<uint> BuildGetAcceptanceMechanismsRequestAsync(
            long timestamp,
            string version = null,
            string submitterDid = null)
        {
            uint requestHandle = 0;
            int errorCode = NativeMethods.indy_vdr_build_get_acceptance_mechanisms_request(
                FfiStr.Create(submitterDid),
                timestamp,
                FfiStr.Create(version),
                ref requestHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }
            return requestHandle;
        }

        public static async Task<uint> BuildAttributeRequest(
            string targetDid,
            string submitterDid = null,
            string hash = null,
            string raw = null,
            string enc = null)
        {
            uint requestHandle = 0;
            int errorCode = NativeMethods.indy_vdr_build_attrib_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(targetDid),
                FfiStr.Create(hash),
                FfiStr.Create(raw),
                FfiStr.Create(enc),
                ref requestHandle);

            Debug.WriteLine("\n\n TEST");

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }

            return requestHandle;
        }

        public static async Task<uint> BuildGetAttributeRequest(
            string targetDid,
            string submitterDid = null,
            string hash = null,
            string raw = null,
            string enc = null)
        {
            uint requestHandle = 0;
            int errorCode = NativeMethods.indy_vdr_build_get_attrib_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(targetDid),
                FfiStr.Create(hash),
                FfiStr.Create(raw),
                FfiStr.Create(enc),
                ref requestHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }

            return requestHandle;
        }

        public static async Task<uint> BuildCredDefRequest(
            string submitterDid,
            string credDef)
        {
            uint requestHandle = 0;
            int errorCode = NativeMethods.indy_vdr_build_cred_def_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(credDef),
                ref requestHandle);


            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }

            return requestHandle;
        }

        public static async Task<uint> BuildCustomRequest(
            string requestJson)
        {
            uint requestHandle = 0;
            int errorCode = NativeMethods.indy_vdr_build_custom_request(
                FfiStr.Create(requestJson),
                ref requestHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }
            return requestHandle;
        }

        public static async Task<uint> BuildDisableAllTxnAuthorAgreementsRequest(
            string submitterDid)
        {
            uint requestHandle = 0;
            int errorCode = NativeMethods.indy_vdr_build_disable_all_txn_author_agreements_request(
                FfiStr.Create(submitterDid),
                ref requestHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }
            return requestHandle;
        }

        public static async Task<uint> BuildGetCredDefRequest(
            string credDefDid,
            string submitterDid = null)
        {
            uint requestHandle = 0;
            int errorCode = NativeMethods.indy_vdr_build_get_cred_def_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(credDefDid),
                ref requestHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }
            return requestHandle;
        }

        public static async Task<uint> BuildGetNymRequest(
            string targetDid,
            string submitterDid = null)
        {
            uint requestHandle = 0;
            int errorCode = NativeMethods.indy_vdr_build_get_nym_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(targetDid),
                ref requestHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }
            return requestHandle;
        }

        public static async Task<uint> BuildGetRevocRegDefRequest(
            string revocRegId,
            string submitterDid = null)
        {
            uint requestHandle = 0;
            int errorCode = NativeMethods.indy_vdr_build_get_revoc_reg_def_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(revocRegId),
                ref requestHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }
            return requestHandle;
        }

        public static async Task<uint> BuildGetRevocRegRequest(
            string revocRegId,
            long timestamp,
            string submitterDid = null)
        {
            uint requestHandle = 0;
            int errorCode = NativeMethods.indy_vdr_build_get_revoc_reg_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(revocRegId),
                timestamp,
                ref requestHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }
            return requestHandle;
        }

        public static async Task<uint> BuildGetRevocRegDeltaRequestAsync(
            string revocRegId,
            long toTS,
            long fromTs = -1,
            string submitterDid = null)
        {
            uint requestHandle = 0;
            int errorCode = NativeMethods.indy_vdr_build_get_revoc_reg_delta_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(revocRegId),
                fromTs,
                toTS,
                ref requestHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }

            return requestHandle;
        }

        public static async Task<uint> BuildGetSchemaRequestAsync(
            string schemaId,
            string submitterDid = null)
        {
            uint requestHandle = 0;
            int errorCode = NativeMethods.indy_vdr_build_get_schema_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(schemaId),
                ref requestHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }

            return requestHandle;
        }

        public static async Task<uint> BuildGetTxnAuthorAgreementRequestAsync(
            string submitterDid = null,
            string data = null)
        {
            uint requestHandle = 0;
            int errorCode = NativeMethods.indy_vdr_build_get_txn_author_agreement_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(data),
                ref requestHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }

            return requestHandle;
        }

        public static async Task<uint> BuildGetTxnRequestAsync(
            int ledgerType,
            int seqNo,
            string submitterDid = null)
        {
            uint requestHandle = 0;
            int errorCode = NativeMethods.indy_vdr_build_get_txn_request(
                FfiStr.Create(submitterDid),
                ledgerType,
                seqNo,
                ref requestHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }

            return requestHandle;
        }

        public static async Task<uint> BuildGetValidatorInfoRequestAsync(
            string submitterDid)
        {
            uint requestHandle = 0;
            int errorCode = NativeMethods.indy_vdr_build_get_validator_info_request(
                FfiStr.Create(submitterDid),
                ref requestHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }

            return requestHandle;
        }

        public static async Task<uint> BuildNymRequestAsync(
            string submitterDid,
            string dest,
            string verkey = null,
            string alias = null,
            string role = null)
        {
            uint requestHandle = 0;
            int errorCode = NativeMethods.indy_vdr_build_nym_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(dest),
                FfiStr.Create(verkey),
                FfiStr.Create(alias),
                FfiStr.Create(role),
                ref requestHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }

            return requestHandle;
        }

        public static async Task<uint> BuildRevocRegDefRequestAsync(
            string submitterDid,
            string revocRegDefJson)
        {
            uint requestHandle = 0;
            int errorCode = NativeMethods.indy_vdr_build_revoc_reg_def_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(revocRegDefJson),
                ref requestHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }

            return requestHandle;
        }

        public static async Task<uint> BuildRevocRegEntryRequestAsync(
            string submitterDid,
            string revocRegDefId,
            string revocRegDefType,
            string revocRegEntryJson)
        {
            uint requestHandle = 0;
            int errorCode = NativeMethods.indy_vdr_build_revoc_reg_entry_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(revocRegDefId),
                FfiStr.Create(revocRegDefType),
                FfiStr.Create(revocRegEntryJson),
                ref requestHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }

            return requestHandle;
        }

        public static async Task<uint> BuildSchemaRequestAsync(
            string submitterDid,
            string schemaJson)
        {
            uint requestHandle = 0;
            int errorCode = NativeMethods.indy_vdr_build_schema_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(schemaJson),
                ref requestHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }
            /*
            string requestJson = "";
            var bodyErrorCode = NativeMethods.indy_vdr_request_get_body(requestHandle, ref requestJson);

            if (bodyErrorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Debug.WriteLine(error);
            }*/

            return requestHandle;
        }

        public static async Task<uint> BuildTxnAuthorAgreementRequestAsync(
            string submitterDid,
            string text,
            string version,
            long ratificationTs,
            long retirementTs)
        {
            uint requestHandle = 0;
            int errorCode = NativeMethods.indy_vdr_build_txn_author_agreement_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(text),
                FfiStr.Create(version),
                ratificationTs,
                retirementTs,
                ref requestHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }

            return requestHandle;
        }

        public static async Task<uint> BuildRichSchemaRequestAsync(
            string submitterDid,
            string rsId,
            string rsContent,
            string rsName,
            string rsVersion,
            string rsType,
            string version)
        {
            uint requestHandle = 0;
            int errorCode = NativeMethods.indy_vdr_build_rich_schema_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(rsId),
                FfiStr.Create(rsContent),
                FfiStr.Create(rsName),
                FfiStr.Create(rsVersion),
                FfiStr.Create(rsType),
                FfiStr.Create(version),
                ref requestHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }

            return requestHandle;
        }

        public static async Task<uint> BuildGetRichSchemaObjectByIdRequestAsync(
            string submitterDid,
            string rsId)
        {
            uint requestHandle = 0;
            int errorCode = NativeMethods.indy_vdr_build_get_rich_schema_object_by_id_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(rsId),
                ref requestHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }

            return requestHandle;
        }

        public static async Task<uint> BuildGetRichSchemaObjectByMetadataRequestAsync(
            string submitterDid,
            string rsType,
            string rsName,
            string rsVersion)
        {
            uint requestHandle = 0;
            int errorCode = NativeMethods.indy_vdr_build_get_rich_schema_object_by_metadata_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(rsType),
                FfiStr.Create(rsName),
                FfiStr.Create(rsVersion),
                ref requestHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }

            return requestHandle;
        }
    }
}
