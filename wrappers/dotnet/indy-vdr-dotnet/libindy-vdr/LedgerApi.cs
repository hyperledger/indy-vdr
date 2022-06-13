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
            string submitter_did,
            string aml,
            string verion,
            string aml_context = null)
        {
            uint request_handle = 0;
            int errorCode = NativeMethods.indy_vdr_build_acceptance_mechanisms_request(
                FfiStr.Create(submitter_did),
                FfiStr.Create(aml),
                FfiStr.Create(verion),
                FfiStr.Create(aml_context),
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
            string submitter_did = null)
        {
            uint request_handle = 0;
            int errorCode = NativeMethods.indy_vdr_build_get_acceptance_mechanisms_request(
                FfiStr.Create(submitter_did),
                timestamp,
                FfiStr.Create(version),
                ref request_handle);

            await ErrorApi.GetCurrentErrorAsync();

            return await Task.FromResult(request_handle);
        }
    }
}
