using System;
using System.Threading.Tasks;
using static indy_vdr_dotnet.models.Structures;

namespace indy_vdr_dotnet.libindy_vdr
{
    public static class RequestApi
    {
        public static async Task<string> PrepareTxnAuthorAgreementAcceptanceAsync(
            string accMechType,
            ulong time,
            string text = null,
            string version = null,
            string taaDigest = null)
        {
            string output = "";
            int errorCode = NativeMethods.indy_vdr_prepare_txn_author_agreement_acceptance(
                FfiStr.Create(text),
                FfiStr.Create(version),
                FfiStr.Create(taaDigest),
                FfiStr.Create(accMechType),
                time,
                ref output);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }

            return output;
        }

        public static async Task RequestFreeAsync(
            uint requestHandle)
        {
            int errorCode = NativeMethods.indy_vdr_request_free(
                requestHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }
        }

        public static async Task<string> RequestGetBodyAsync(
            uint requestHandle)
        {
            string output = "";
            int errorCode = NativeMethods.indy_vdr_request_get_body(
                requestHandle,
                ref output);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }
            return output;
        }

        public static async Task<string> RequestGetSignatureInputAsync(
            uint requestHandle)
        {
            string output = "";
            int errorCode = NativeMethods.indy_vdr_request_get_signature_input(
                requestHandle,
                ref output);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }
            return output;
        }

        public static async Task RequestSetEndorserAsync(
            uint requestHandle,
            string endorser)
        {
            int errorCode = NativeMethods.indy_vdr_request_set_endorser(
                requestHandle,
                FfiStr.Create(endorser));

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }
        }

        public static async Task RequestSetMultiSignatureAsync(
            uint requestHandle,
            string identifier,
            string signatureJson)
        {
            int errorCode = NativeMethods.indy_vdr_request_set_multi_signature(
                requestHandle,
                FfiStr.Create(identifier),
                ByteBuffer.Create(signatureJson));

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }
        }

        public static async Task RequestSetSigantureAsync(
            uint requestHandle,
            string signature)
        {
            int errorCode = NativeMethods.indy_vdr_request_set_signature(
                requestHandle,
                ByteBuffer.Create(signature));

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }
        }

        public static async Task RequestSetTxnAuthorAgreementAcceptanceAsync(
            uint requestHandle,
            string acceptance)
        {
            int errorCode = NativeMethods.indy_vdr_request_set_txn_author_agreement_acceptance(
                requestHandle,
                FfiStr.Create(acceptance));

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }
        }
    }
}
