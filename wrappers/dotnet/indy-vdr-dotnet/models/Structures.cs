using System;
using System.Runtime.InteropServices;
using System.Text;

namespace indy_vdr_dotnet.models
{
    public class Structures
    {
        [StructLayout(LayoutKind.Sequential)]
        public struct FfiStr
        {
            public IntPtr data;

            public static FfiStr Create(string arg)
            {
                FfiStr FfiString = new();
                FfiString.data = new IntPtr();
                if (arg != null)
                {
                    FfiString.data = Marshal.StringToCoTaskMemUTF8(arg);
                }
                return FfiString;
            }
        }

        [StructLayout(LayoutKind.Sequential)]
        public unsafe struct ByteBuffer
        {
            public long len;
            public byte* value;

            public static ByteBuffer Create(string json)
            {
                UTF8Encoding decoder = new UTF8Encoding(true, true);
                byte[] bytes = new byte[json.Length];
                decoder.GetBytes(json, 0, json.Length, bytes, 0);
                ByteBuffer buffer = new();
                buffer.len = (uint)json.Length;
                fixed (byte* bytebuffer_p = &bytes[0])
                {
                    buffer.value = bytebuffer_p;
                }
                return buffer;
            }
        }
    }
}
