using FluentAssertions;
using indy_vdr_dotnet.libindy_vdr;
using NUnit.Framework;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace indy_vdr_dotnet_tests.libindy_vdr
{
    public class ModTests
    {
        [Test]
        [TestCase(TestName = "GetVersionAsync returns a string that is not empty.")]
        public async Task GetVersion()
        {
            //Arrange

            //Act
            string actual = await Mod.GetVersionAsync();

            //Assert
            actual.Should().NotBeEmpty();
            Console.WriteLine(actual);
        }
    }
}
