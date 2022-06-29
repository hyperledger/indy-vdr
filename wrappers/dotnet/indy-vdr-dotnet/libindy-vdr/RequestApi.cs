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
            string agreementAcceptance = "";
            int errorCode = NativeMethods.indy_vdr_prepare_txn_author_agreement_acceptance(
                FfiStr.Create(text),
                FfiStr.Create(version),
                FfiStr.Create(taaDigest),
                FfiStr.Create(accMechType),
                time,
                ref agreementAcceptance);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }

            return agreementAcceptance;
        }

        public static async Task RequestFreeAsync(
            IntPtr requestHandle)
        {
            int errorCode = NativeMethods.indy_vdr_request_free(
                requestHandle);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }
        }

        public static async Task<string> RequestGetBodyAsync(
            IntPtr requestHandle)
        {
            string body = "";
            int errorCode = NativeMethods.indy_vdr_request_get_body(
                requestHandle,
                ref body);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }
            return body;
        }

        public static async Task<string> RequestGetSignatureInputAsync(
            IntPtr requestHandle)
        {
            string signature = "";
            int errorCode = NativeMethods.indy_vdr_request_get_signature_input(
                requestHandle,
                ref signature);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }
            return signature;
        }

        public static async Task RequestSetEndorserAsync(
            IntPtr requestHandle,
            string endorser)
        {
            int errorCode = NativeMethods.indy_vdr_request_set_endorser(
                requestHandle,
                FfiStr.Create(endorser));

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }
        }

        public static async Task RequestSetMultiSignatureAsync(
            IntPtr requestHandle,
            string identifier,
            string signature)
        {
            int errorCode = NativeMethods.indy_vdr_request_set_multi_signature(
                requestHandle,
                FfiStr.Create(identifier),
                ByteBuffer.Create(signature));

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }
        }

        public static async Task RequestSetSigantureAsync(
            IntPtr requestHandle,
            string signature)
        {
            int errorCode = NativeMethods.indy_vdr_request_set_signature(
                requestHandle,
                ByteBuffer.Create(signature));

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }
        }

        public static async Task RequestSetTxnAuthorAgreementAcceptanceAsync(
            IntPtr requestHandle,
            string agreementAcceptance)
        {
            int errorCode = NativeMethods.indy_vdr_request_set_txn_author_agreement_acceptance(
                requestHandle,
                FfiStr.Create(agreementAcceptance));

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }
        }
    }
}
