using Newtonsoft.Json;
using System.Collections.Generic;

namespace indy_vdr_dotnet.models
{
    public class Schema
    {
        public uint Handle { get; set; }
        [JsonProperty("id")]
        public string Id { get; set; }
        [JsonProperty("name")]
        public string Name { get; set; }
        [JsonProperty("version")]
        public string Version { get; set; }
        [JsonProperty("attrNames")]
        public HashSet<string> AttrNames { get; set; }
        [JsonProperty("ver")]
        public string Ver { get; set; }
        public uint SeqNo { get; set; }
    }
}
