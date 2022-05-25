using FluentAssertions;
using indy_vdr_dotnet.libindy_vdr;
using NUnit.Framework;
using System.Threading.Tasks;

namespace indy_vdr_dotnet_tests.libindy_vdr
{
    internal class Errortests
    {
        [Test]
        [TestCase(TestName = "GetCurrentErrorAsync returns the json of an empty error.")]
        public async Task GetCurrentError()
        {
            //Arrange

            //Act
            string expected = "{\"code\":0,\"message\":null,\"extra\":null}";
            string actual = await Error.GetCurrentErrorAsync();

            //Assert
            actual.Should().Be(expected);
        }
    }
}
