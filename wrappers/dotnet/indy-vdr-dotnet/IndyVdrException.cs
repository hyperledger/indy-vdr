using Newtonsoft.Json;
using System;
using System.Collections.Generic;

namespace indy_vdr_dotnet
{
    public class IndyVdrException : Exception
    {
        public IndyVdrException(string message) : base(message)
        {
        }

        public IndyVdrException(string message, Exception inner) : base(message, inner)
        {
        }

        public static IndyVdrException FromSdkError(string message)
        {
            string msg = JsonConvert.DeserializeObject<Dictionary<string, string>>(message)["message"];
            string errCode = JsonConvert.DeserializeObject<Dictionary<string, string>>(message)["code"];
            string extra = JsonConvert.DeserializeObject<Dictionary<string, string>>(message)["extra"];
            int errCodeInt;
            if (int.TryParse(errCode, out errCodeInt))
            {
                return new IndyVdrException(
                    $"'{ErrorCodeConverter.ToErrorCodeString((ErrorCode)errCodeInt)}' error occured with ErrorCode '{errCode}' and extra: '{extra}': {msg}.");
            }
            else
            {
                return new IndyVdrException("An unknown error code was received.");
            }     
        }
    }
}
