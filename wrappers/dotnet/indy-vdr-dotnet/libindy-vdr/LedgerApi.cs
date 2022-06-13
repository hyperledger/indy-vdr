using System.Diagnostics;
using System.Threading.Tasks;
using static indy_vdr_dotnet.models.Structures;

namespace indy_vdr_dotnet.libindy_vdr
{
    public static class LedgerApi
    {
        public static async Task<uint> BuildSchemaRequest(
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
