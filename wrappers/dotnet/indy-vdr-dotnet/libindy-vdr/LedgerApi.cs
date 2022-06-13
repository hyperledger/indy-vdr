using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Text;
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
            uint request_handle = 0;
            int errorCode = NativeMethods.indy_vdr_build_acceptance_mechanisms_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(aml),
                FfiStr.Create(verion),
                FfiStr.Create(amlContext),
                ref request_handle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Debug.WriteLine(error);
            }

            return await Task.FromResult(request_handle);
        }

        public static async Task<uint> BuildGetAcceptanceMechanismsRequestAsync(
            long timestamp,
            string version = null,
            string submitterDid = null)
        {
            uint request_handle = 0;
            int errorCode = NativeMethods.indy_vdr_build_get_acceptance_mechanisms_request(
                FfiStr.Create(submitterDid),
                timestamp,
                FfiStr.Create(version),
                ref request_handle);

            if (errorCode != 0)
            {
                await ErrorApi.GetCurrentErrorAsync();
            }
            return await Task.FromResult(request_handle);
        }

        public static async Task<uint> BuildAttributeRequest(
            string targetDid,
            string submitterDid = null,
            string hash = null,
            string raw = null,
            string enc = null)
        {
            uint request_handle = 0;
            int errorCode = NativeMethods.indy_vdr_build_attrib_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(targetDid),
                FfiStr.Create(hash),
                FfiStr.Create(raw),
                FfiStr.Create(enc),
                ref request_handle);

            Debug.WriteLine("\n\n TEST");

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Debug.WriteLine(error);
            }

            return await Task.FromResult(request_handle);
        }

        public static async Task<uint> BuildGetAttributeRequest(
            string targetDid,
            string submitterDid = null,
            string hash = null,
            string raw = null,
            string enc = null)
        {
            uint request_handle = 0;
            int errorCode = NativeMethods.indy_vdr_build_get_attrib_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(targetDid),
                FfiStr.Create(hash),
                FfiStr.Create(raw),
                FfiStr.Create(enc),
                ref request_handle);

            if (errorCode != 0)
            {
                await ErrorApi.GetCurrentErrorAsync();
            }

            return await Task.FromResult(request_handle);
        }

        public static async Task<uint> BuildCredDefRequest(
            string submitterDid,
            string credDef)
        {
            uint request_handle = 0;
            int errorCode = NativeMethods.indy_vdr_build_cred_def_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(credDef),
                ref request_handle);


            if (errorCode != 0)
            {
                await ErrorApi.GetCurrentErrorAsync();
            }

            return await Task.FromResult(request_handle);
        }


        public static async Task<uint> BuildSchemaRequestAsync(
            string submitterDid,
            string schema)
        {
            uint requestHandle = 0;
            int errorCode = NativeMethods.indy_vdr_build_schema_request(
                FfiStr.Create(submitterDid),
                FfiStr.Create(schema),
                ref requestHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Debug.WriteLine(error);
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

        public static async Task<uint> BuildTxnAuthorAgreementRequest(
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
                Debug.WriteLine(error);
            }

            return requestHandle;
        }

        public static async Task<uint> BuildRichSchemaRequest(
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
                Debug.WriteLine(error);
            }

            return requestHandle;
        }

        public static async Task<uint> BuildGetRichSchemaObjectByIdRequest(
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
                Debug.WriteLine(error);
            }

            return requestHandle;
        }

        public static async Task<uint> BuildGetRichSchemaObjectByMetadataRequest(
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
                Debug.WriteLine(error);
            }

            return requestHandle;
        }
    }
}
