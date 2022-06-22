using FluentAssertions;
using indy_vdr_dotnet.libindy_vdr;
using Newtonsoft.Json;
using NUnit.Framework;
using System;
using System.IO;
using System.Threading.Tasks;

namespace indy_vdr_dotnet_tests.libindy_vdr
{
    public class ModApiTests
    {
        [Test]
        [TestCase(TestName = "GetVersionAsync returns a string that is not empty.")]
        public async Task GetVersion()
        {
            //Arrange

            //Act
            string actual = await ModApi.GetVersionAsync();

            //Assert
            actual.Should().NotBeEmpty();
        }

        [Test]
        [TestCase(TestName = "SetConfig sets the pool config.")]
        public async Task SetConfig()
        {
            //Arrange
            string currentDirectory = AppDomain.CurrentDomain.BaseDirectory;
            string genesisFile = Path.Combine(currentDirectory, @"..\..\..\Resources\ew_builder");
            string _genesisFilePath = Path.GetFullPath(genesisFile);

            //Act
            string testConfigJson = JsonConvert.SerializeObject(new
            {
                protocol_version = "Node1_4",
                freshness_threshold = 1,
                ack_timeout = 10000,
                reply_timeout = 10000,
                conn_request_limit = 1l,
                conn_active_timeout = 10000,
                request_read_nodes = 1l,
                socks_proxy = "proxy1.intranet.company.com:1080"
            });
            int errorCode = await ModApi.SetConfigAsync(testConfigJson);

            //Assert
            errorCode.Should().Be(0);
        }

        [Test]
        [TestCase(TestName = "SetDefaultLogger does not throw an exception.")]
        public async Task SetDefaultLogger()
        {
            //Arrange

            //Act
            int errorCode = await ModApi.SetDefaultLoggerAsync();

            //Assert
            errorCode.Should().Be(0);
        }

        [Test]
        [TestCase(TestName = "SetSocksProxy sets the socks proxy.")]
        public async Task SetSocksProxy()
        {
            //Arrange
            string testSocksProxy = "proxy1.intranet.company.com:1080";

            //Act
            int errorCode = await ModApi.SetSocksProxyAsync(testSocksProxy);

            //Assert
            errorCode.Should().Be(0);
        }

        [Test]
        [TestCase(TestName = "SetProtocolVersion sets the protocol version.")]
        public async Task SetProtocolVersion()
        {
            //Arrange

            //Act
            int errorCode = await ModApi.SetProtocolVersionAsync(2);

            //Assert
            errorCode.Should().Be(0);
        }
    }
}
