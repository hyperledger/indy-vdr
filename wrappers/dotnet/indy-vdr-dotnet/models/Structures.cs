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
                FfiStr ffiString = new FfiStr();
                ffiString.data = new IntPtr();
                if (arg != null)
                {
                    ffiString.data = Marshal.StringToCoTaskMemAnsi(arg);
                }
                return ffiString;
            }
        }

        [StructLayout(LayoutKind.Sequential)]
        public unsafe struct ByteBuffer
        {
            public long len;
            public IntPtr value;

            public static ByteBuffer Create(string json)
            {
                ByteBuffer buffer = new ByteBuffer();
                if (!string.IsNullOrEmpty(json))
                {
                UTF8Encoding decoder = new UTF8Encoding(true, true);
                byte[] bytes = new byte[json.Length];
                    _ = decoder.GetBytes(json, 0, json.Length, bytes, 0);
                    buffer.len = json.Length;
                fixed (byte* bytebuffer_p = &bytes[0])
                {
                        buffer.value = new IntPtr(bytebuffer_p);
                    }
                }
                else
                {
                    buffer.len = 0;
                    buffer.value = new IntPtr();
                }
                return buffer;
            }

            public static ByteBuffer Create(byte[] bytes)
            {
                ByteBuffer buffer = new ByteBuffer();
                buffer.len = bytes != null ? bytes.Length : 0;

                if (buffer.len > 0 && bytes != null)
                {
                fixed (byte* bytebuffer_p = &bytes[0])
                {
                        buffer.value = new IntPtr(bytebuffer_p);
                    }
                }
                else
                {
                    buffer.value = new IntPtr();
                }

                return buffer;
            }
        }
    }
}
