require "json"

package = JSON.parse(File.read(File.join(__dir__, "package.json")))

Pod::Spec.new do |s|
  s.name         = "react-native-indy-vdr"
  s.version      = package["version"]
  s.summary      = package["description"]
  s.homepage     = package["homepage"]
  s.license      = package["license"]
  s.authors      = package["author"]

  s.platforms    = { :ios => "10.0" }
  s.source       = { :git => "https://github.com/hyperledger/indy-vdr.git", 
                     :tag => "#{s.version}" }

  source_base    = "wrappers/javascript/react-native-indy-vdr/"

  s.source_files = [
    source_base + "ios/**/*.{h,m,mm}", 
    source_base + "cpp/**/*.{h,cpp}"
  ]

  s.dependency "React-Core"
end
