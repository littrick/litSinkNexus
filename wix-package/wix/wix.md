```sh
dotnet tool install -g wix
```
```sh
wix extension add WixToolset.UI.wixext WixToolset.Util.wixext
```
```sh
wix build main.wxs -ext WixToolset.UI.wixext -ext WixToolset.Util.wixext -culture zh-CN
```
```sh
wix build ./*.wxs ./*.wxl -ext WixToolset.UI.wixext -culture zh-CN -o installer.msi
```