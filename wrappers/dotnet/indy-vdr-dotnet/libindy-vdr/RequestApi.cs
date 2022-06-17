using System;
using System.Threading.Tasks;
using static indy_vdr_dotnet.models.Structures;

namespace indy_vdr_dotnet.libindy_vdr
{
    public static class RequestApi
    {
        public static async Task<string> PrepareTxnAuthorAgreementAcceptance(
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

        public static async Task RequestFree(
            uint requestHandle)
        {
            string output = "";
            int errorCode = NativeMethods.indy_vdr_request_free(
                requestHandle);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }
        }

        public static async Task<string> RequestGetBody(
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

        public static async Task<string> RequestGetSignatureInput(
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

        public static async Task RequestSetEndorser(
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

        public static async Task RequestSetMultiSignature(
            uint requestHandle,
            string identifier,
            ByteBuffer signature)
        {
            int errorCode = NativeMethods.indy_vdr_request_set_multi_signature(
                requestHandle,
                FfiStr.Create(identifier),
                signature);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }
        }

        public static async Task RequestSetSiganture(
            uint requestHandle,
            ByteBuffer signature)
        {
            int errorCode = NativeMethods.indy_vdr_request_set_signature(
                requestHandle,
                signature);

            if (errorCode != 0)
            {
                string error = "";
                NativeMethods.indy_vdr_get_current_error(ref error);
                Console.WriteLine(error);
            }
        }

        public static async Task RequestSetTxnAuthorAgreementAcceptance(
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
