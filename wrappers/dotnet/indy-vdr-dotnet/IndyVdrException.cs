using Newtonsoft.Json;
using System;
using System.Collections.Generic;

namespace indy_vdr_dotnet
{
    public class IndyVdrException : Exception
    {
        public ErrorCode ErrorCode { get; set; }
        public IndyVdrException(string message) : base(message)
        {
        }

        public IndyVdrException(string message, Exception inner) : base(message, inner)
        {
        }

        public IndyVdrException(string message, ErrorCode code) : base(message)
        {
            ErrorCode = code;
        }

        public static IndyVdrException FromSdkError(string message)
        {
            string msg = JsonConvert.DeserializeObject<Dictionary<string, string>>(message)["message"];
            string errCode = JsonConvert.DeserializeObject<Dictionary<string, string>>(message)["code"];
            return int.TryParse(errCode, out int errCodeInt)
                ? new IndyVdrException(
                    $"'{((ErrorCode)errCodeInt).ToErrorCodeString()}' error occured with ErrorCode '{errCode}' : {msg}.", (ErrorCode)errCodeInt)
                : new IndyVdrException("An unknown error code was received.", (ErrorCode)errCodeInt);
        }
    }
}
