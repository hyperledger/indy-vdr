using System.Threading.Tasks;
using static indy_vdr_dotnet.models.Structures;

namespace indy_vdr_dotnet.libindy_vdr
{
    public static class ModApi
    {
        public static async Task<int> SetConfigAsync(
            string config)
        {
            int errorCode = NativeMethods.indy_vdr_set_config(
                FfiStr.Create(config));

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }
            return errorCode;
        }

        public static async Task<int> SetDefaultLoggerAsync()
        {
            int errorCode = NativeMethods.indy_vdr_set_default_logger();

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }
            return errorCode;
        }

        public static async Task<int> SetProtocolVersionAsync(long version)
        {
            int errorCode = NativeMethods.indy_vdr_set_protocol_version(version);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }
            return errorCode;
        }

        public static async Task<int> SetSocksProxyAsync(string socks_proxy)
        {
            int errorCode = NativeMethods.indy_vdr_set_socks_proxy(FfiStr.Create(socks_proxy));
            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }
            return errorCode;
        }

        public static async Task<string> GetVersionAsync()
        {
            string version = NativeMethods.indy_vdr_version();

            return version;
        }
    }
}