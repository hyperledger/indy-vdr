using System.Runtime.InteropServices;

namespace indy_vdr_dotnet.libindy_vdr
{
    internal static class NativeMethods
    {
        #region Mod
        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern void indy_vdr_set_default_logger();

        [DllImport(Consts.LIBINDY_VDR_NAME, CharSet = CharSet.Ansi, BestFitMapping = false, ThrowOnUnmappableChar = true)]
        internal static extern string indy_vdr_version();
        #endregion
    }
}