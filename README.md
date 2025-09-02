保存機能をどうしても作りたいので、勝手にフォークして勝手に改造します（未完成につきダウンロードはオススメしません。）



# NordGeits Note

Simply said, this is an MMD plugin that allows you to set keybinds.
This is a translation effort, primarily using DeepL and testing what the shortcuts do. If you actually do know japanese, contributing is highly appreciated.
There are no releases yet.
With the exception of the copy of MikuMikuDance included in the folders, this program has an MIT license, allowing anyone to modify and tinker with the program.
This translation effort is by default licensed with this too but it's like a fork and idc

There's only two json files to edit to translate the main content.

# MMAccel

MMDにショートカット+αを追加するツール

## インストール

d3d9.dllとMMAccelフォルダをMikuMikuDance.exeと同じフォルダにコピーしてください。
MMAccelのd3d9.dllでMMEのd3d9.dllを上書きしてもMMEを使用できます。

インストールが成功すれば、MMDにMMAccelというメニューが現れます。

## 使い方

MMAccelメニューを開いて「キー設定」をクリックすると、キー設定のウィンドウが表示されます。

目的の動作を探して項目をダブルクリックすると入力待ち状態となります。
ここで割り当てたいショートカットキー押した後に

* クリックで確定
* 右クリックでキャンセル

となります。確定した時点でファイルに保存されてMMDに適用されます。

キーの解除は解除したい項目を右クリックでメニューを出して「解除」をクリックしてください。

設定したキーはMMAccelフォルダ内のkey_map.jsonに保存されます。

## 設定

### タイマーの精度を上げる

FPS無制限時のフレームレートが上がったりします。
詳しく言えば`timeBeginPeriod(1)`を呼び出してタイマーの精度を上げます。

### クリックで入力状態を解除

数値などを入力しているときに別の場所をクリックすると入力状態が解除されるようになります。

## 注意事項

### 以前のバージョンのkey_map.txt

2.0.0からkey_map.jsonになりkey_map.txtは使われませんが、
キー設定のウィンドウにkey_map.txtをドラッグアンドドロップすることで取り込むことができます。

### MMEとの併用

MMAccelのd3d9.dllではMMEを動作させることが出来ますが、MMEのd3d9.dllではMMAccelを動作させることが出来ません。
また、MMEにおいてCtrl+Shift+Eが「エフェクト使用」で使われており、MMAccelで割り当てると割り当てた動作が行われるのと同時にMMEの「エフェクト使用」をクリックした状態になります。

### MMPlusとの併用

**MMPlusはMMAccelとの併用を動作保証外としているので、MMPlusの方にMMAccelや併用についての問い合わせをしないでください。**

MMAccelを入れるとMMPlusのキーボードショートカットは動かなくなります。

### MMPluginへの対応

2021/05/06の時点でMMPluginへの対応はありません。

### MMDのバージョン

動作確認をMMD v9.32 64bitで行っています。それ以前のMMDでの動作は保証できません。
また、32bit版はありません。

## アンインストール

d3d9.dllとMMAccelフォルダを捨ててください。
MMEを引き続き使う場合はMMEのd3d9.dllを入れ戻してください。

## リポジトリ

https://github.com/LNSEAB/MMAccel
  
## ライセンス

MIT License (リポジトリのMikuMikuDance_v932x64は除く）

使用しているクレートのライセンスはMMAccelフォルダ内のLICENSE-DEPENDENCIESにあります。

## 謝辞

MMEについてご教示くださいました舞力介入Pに御礼申し上げます。

-------------------------------------------------

Copyright (c) 2021 LNSEAB

twitter : http://twitter.com/lnseab
