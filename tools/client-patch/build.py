# Builds patched AS3 sources for Main.swf startup speed.
# usage: python -I build.py <decompiled scripts dir> <out dir> [trace]
import re, sys, os
src, out = sys.argv[1], sys.argv[2]
TRACE = len(sys.argv) > 3
os.makedirs(out, exist_ok=True)
files = {}

def rd(p):
    return open(os.path.join(src, p), encoding='utf-8').read()

def rep(s, old, new, count=1):
    n = s.count(old)
    if n != count:
        sys.exit('pattern count %d != %d: %r' % (n, count, old[:90]))
    return s.replace(old, new)

def add_imports(s, *imps):
    i = s.index('\n   import ')
    return s[:i] + ''.join('\n   import %s;' % x for x in imps) + s[i:]

def trace_lines(s, fn_sig, prefix):
    a = s.index(fn_sig); b = s.index('\n      }\n', a)
    out_l = []
    for l in s[a:b].split('\n'):
        out_l.append(l)
        st = l.strip()
        if l.startswith('         ') and not l.startswith('          ') and st.endswith(';') and not st.startswith('var '):
            tag = re.sub(r'[^A-Za-z0-9_.]', '', st[:50])
            out_l.append('         ExternalInterface.call("console.log","%s %s " + (getTimer() - _kpt)); _kpt = getTimer();' % (prefix, tag))
    out_l.insert(2, '         var _kpt:int = getTimer();')
    s = s[:a] + '\n'.join(out_l) + s[b:]
    return add_imports(s, 'flash.external.ExternalInterface', 'flash.utils.getTimer')

# 1. XmlManager: every config field parsed on first read instead of all at startup.
s = rd('game/manager/XmlManager.as')
a = s.index('      public static function initXML() : void')
b = s.index('\n      }\n', a)
pat = re.compile(r'^\s*(\w+) = ((?:XmlManager\.)?getXml\("[^"]+"\)|initXmlToObj(?:ForKeyInt)?\((?:getXml\("[^"]+"\)|\w+),"[^"]+"\));$')
lazy, keep = {}, []
for line in s[a:b].split('\n'):
    m = pat.match(line)
    if m and m.group(1) not in lazy:
        lazy[m.group(1)] = m.group(2)
    else:
        keep.append(line)
s = s[:a] + '\n'.join(keep) + s[b:]
for v, expr in lazy.items():
    d = re.search(r'      public static var %s:(\w+);\n' % v, s)
    if not d:
        sys.exit('no plain declaration for ' + v)
    t = d.group(1)
    s = s.replace(d.group(0),
        '      private static var _lz_%s:%s;\n\n'
        '      public static function get %s() : %s\n      {\n'
        '         if(_lz_%s == null)\n         {\n            _lz_%s = %s;\n         }\n'
        '         return _lz_%s;\n      }\n\n'
        '      public static function set %s(param1:%s) : void\n      {\n'
        '         _lz_%s = param1;\n      }\n' % (v, t, v, t, v, v, expr, v, v, t, v))
s = rep(s, '''      public static function clearXml(param1:String) : void''', '''      public static function getRawXml(param1:String) : String
      {
         return xmlReader.getRaw(param1);
      }

      public static function clearXml(param1:String) : void''')
files['game.manager.XmlManager'] = s

# 2. XmlReader: keep zip entries, parse on first getXml; getRaw gives the text unparsed.
s = rd('game/manager/com/XmlReader.as')
s = rep(s, '''      private var xmlPool:Dictionary;
''', '''      private var xmlPool:Dictionary;

      private var filePool:Dictionary = new Dictionary();
''')
s = rep(s, '''         if(this._isPackage)
         {
            return this.xmlPool[param1.toLowerCase()];
         }
         return Core.dataLib.getXML''', '''         var _loc2_:String = null;
         var _loc3_:FZipFile = null;
         if(this._isPackage)
         {
            _loc2_ = param1.toLowerCase();
            _loc3_ = this.filePool[_loc2_];
            if(_loc3_ != null)
            {
               delete this.filePool[_loc2_];
               this.xmlPool[_loc2_] = new XML(_loc3_.content);%s
               _loc3_.setContent(null,false);
            }
            return this.xmlPool[_loc2_];
         }
         return Core.dataLib.getXML''')
PARSE = '''               delete this.filePool[_loc2_];
               this.xmlPool[_loc2_] = new XML(_loc3_.content);%s'''
s = rep(s, PARSE, PARSE.replace('%s', '') if not TRACE else '''               var _kpx:int = getTimer();
               delete this.filePool[_loc2_];
               this.xmlPool[_loc2_] = new XML(_loc3_.content);
               ExternalInterface.call("console.log","KPX " + _loc2_ + " " + (getTimer() - _kpx));''')
if TRACE:
    s = add_imports(s, 'flash.external.ExternalInterface', 'flash.utils.getTimer')
s = rep(s, '''      public function clearXml(param1:String) : void''', '''      public function getRaw(param1:String) : String
      {
         var _loc2_:FZipFile = this.filePool[param1.toLowerCase()];
         if(_loc2_ != null)
         {
            return _loc2_.getContentAsString(false);
         }
         return null;
      }

      public function clearXml(param1:String) : void''')
s = rep(s, '''            delete this.xmlPool[param1.toLowerCase()];
''', '''            delete this.xmlPool[param1.toLowerCase()];
            delete this.filePool[param1.toLowerCase()];
''')
s = rep(s, '''               this.addXml(_loc6_.toLowerCase(),_loc4_.content);
               _loc4_.setContent(null,false);''', '''               this.filePool[_loc6_.toLowerCase()] = _loc4_;''')
files['game.manager.com.XmlReader'] = s

# 3. SkillAndBuffXMLManager: skill.xml indexed as text, one <skill/> parsed per lookup.
s = rd('game/data/skill/SkillAndBuffXMLManager.as')
s = rep(s, '''         var _loc2_:int = _buff.children().length();
         var _loc4_:int = 0;
         while(_loc4_ < _loc2_)
         {
            _loc3_ = _buff.children()[_loc4_];''', '''         var _loc5_:XMLList = _buff.children();
         var _loc2_:int = _loc5_.length();
         var _loc4_:int = 0;
         while(_loc4_ < _loc2_)
         {
            _loc3_ = _loc5_[_loc4_];''')
s = rep(s, '''      private static var skillMap:Object;
''', '''      private static var skillMap:Object;

      private static var _skillSrc:String;

      private static var _skillIdx:Object;
''')
s = rep(s, '''         if(_skill == null)
''', '''         if(_skill == null && _skillSrc == null)
''')
s = rep(s, '''            _loc2_ = skillMap[param1];''', '''            _loc2_ = skillNode(param1);''')
s = rep(s, '''      public function get skill() : XML
      {
         return _skill;''', '''      public function get skill() : XML
      {
         if(_skill == null && _skillSrc != null)
         {
            _skill = XmlManager.skill;
         }
         return _skill;''')
s = rep(s, '''      public static function iniBuff(param1:XML) : void''', '''      public static function iniSkillRaw(param1:String) : void
      {
         var _loc3_:int = 0;
         var _loc4_:int = 0;
         _skillSrc = param1;
         skillMap = new Object();
         _skillIdx = new Object();
         var _loc2_:int = param1.indexOf("<skill id=\\"");
         while(_loc2_ != -1)
         {
            _loc3_ = param1.indexOf("/>",_loc2_) + 2;
            _loc4_ = param1.indexOf("\\"",_loc2_ + 11);
            _skillIdx[uint(param1.substring(_loc2_ + 11,_loc4_))] = [_loc2_,_loc3_];
            _loc2_ = param1.indexOf("<skill id=\\"",_loc3_);
         }
      }

      private static function skillNode(param1:uint) : XML
      {
         var _loc3_:Array = null;
         var _loc2_:XML = skillMap[param1];
         if(_loc2_ == null && _skillIdx != null)
         {
            _loc3_ = _skillIdx[param1];
            if(_loc3_ != null)
            {
               _loc2_ = new XML(_skillSrc.substring(_loc3_[0],_loc3_[1]));
               skillMap[param1] = _loc2_;
            }
         }
         return _loc2_;
      }

      public static function iniBuff(param1:XML) : void''')
if 'import game.manager.XmlManager;' not in s:
    s = add_imports(s, 'game.manager.XmlManager')
files['game.data.skill.SkillAndBuffXMLManager'] = s

# 4. HeroDataUtil: O(n^2) loops hoisted; startup monster templates merged per htid on first lookup.
s = rd('game/data/group/HeroDataUtil.as')
s = rep(s, '''         var _loc5_:XML = null;
         if(XmlManager.zhuanzhiConf)
         {
            _loc3_ = uint(XmlManager.zhuanzhiConf.children().length());
            _loc4_ = 0;
            while(_loc4_ < _loc3_)
            {
               _loc5_ = XmlManager.zhuanzhiConf.children()[_loc4_];''', '''         var _loc5_:XML = null;
         var _loc6_:XMLList = null;
         if(XmlManager.zhuanzhiConf)
         {
            _loc6_ = XmlManager.zhuanzhiConf.children();
            _loc3_ = uint(_loc6_.length());
            _loc4_ = 0;
            while(_loc4_ < _loc3_)
            {
               _loc5_ = _loc6_[_loc4_];''')
s = rep(s, '''            return XmlManager.zhuanzhiConf.children()[param1];''', '''            return _loc6_[param1];''')
s = rep(s, '''         var _loc2_:XML = XmlManager.rebirth;
         if(_loc2_)
         {
            _loc3_ = _loc2_.children().length();
            _loc4_ = 0;
            while(_loc4_ < _loc3_)
            {
               if(int(_loc2_.children()[_loc4_].@num) == param1)
               {
                  return int(_loc2_.children()[_loc4_].@level);''', '''         var _loc2_:XML = XmlManager.rebirth;
         var _loc5_:XMLList = null;
         if(_loc2_)
         {
            _loc5_ = _loc2_.children();
            _loc3_ = _loc5_.length();
            _loc4_ = 0;
            while(_loc4_ < _loc3_)
            {
               if(int(_loc5_[_loc4_].@num) == param1)
               {
                  return int(_loc5_[_loc4_].@level);''')
s = rep(s, '_map[param1]', 'mapGet(param1)', 13)
s = rep(s, '            if(_map[_loc5_] == null)', '            if(mapGet(_loc5_) == null)')
s = rep(s, '''         var _loc2_:uint = uint(_materialToHtidMap[param1]);
''', '''         if(_monOrder != null && param1 == "")
         {
            ensureAllMonsters();
         }
         var _loc2_:uint = uint(_materialToHtidMap[param1]);
''')
s = rep(s, '''      public static var monstersHash:Object;
''', '''      public static var monstersHash:Object;

      private static var _monSrc:String;

      private static var _monIdx:Object;

      private static var _monDataSrc:String;

      private static var _monDataIdx:Object;

      private static var _monOrder:Array;

      private static var _monAction:Object;
''')
s = rep(s, '''      public static function initMonsterDictionaryData(''', '''      private static function indexRaw(param1:String, param2:String, param3:Boolean, param4:Array) : Object
      {
         var _loc6_:int = 0;
         var _loc7_:int = 0;
         var _loc8_:int = 0;
         var _loc9_:String = null;
         var _loc5_:Object = new Object();
         var _loc10_:int = param1.indexOf(param2);
         while(_loc10_ != -1)
         {
            _loc6_ = param1.indexOf("/>",_loc10_) + 2;
            _loc7_ = param1.indexOf(" htid=\\"",_loc10_);
            if(_loc7_ != -1 && _loc7_ < _loc6_)
            {
               _loc8_ = param1.indexOf("\\"",_loc7_ + 7);
               _loc9_ = param1.substring(_loc7_ + 7,_loc8_);
               if(!param3 || _loc5_[_loc9_] == null)
               {
                  if(param4 != null && _loc5_[_loc9_] == null)
                  {
                     param4.push(_loc9_);
                  }
                  _loc5_[_loc9_] = [_loc10_,_loc6_];
               }
            }
            _loc10_ = param1.indexOf(param2,_loc6_);
         }
         return _loc5_;
      }

      public static function initMonsterDictionaryRaw(param1:String, param2:String) : void
      {
         _monSrc = param1;
         _monDataSrc = param2;
         _monOrder = [];
         _monAction = new Object();
         _monIdx = indexRaw(param1,"<monsters_tmpl ",true,_monOrder);
         _monDataIdx = indexRaw(param2,"<monsters ",false,null);
      }

      private static function resolveMonster(param1:String) : XML
      {
         var _loc4_:XML = null;
         var _loc5_:XML = null;
         var _loc6_:XML = null;
         var _loc2_:Array = _monIdx[param1];
         if(_loc2_ == null)
         {
            return null;
         }
         delete _monIdx[param1];
         var _loc3_:Array = _monDataIdx[param1];
         if(_loc3_ == null)
         {
            return null;
         }
         _loc4_ = new XML(_monSrc.substring(_loc2_[0],_loc2_[1]));
         _loc5_ = new XML(_monDataSrc.substring(_loc3_[0],_loc3_[1]));
         for each(_loc6_ in _loc5_.attributes())
         {
            if(_loc6_.toString() != "")
            {
               _loc4_["@" + _loc6_.name()] = _loc6_;
            }
         }
         _loc4_.@htid = param1;
         _map[param1] = _loc4_;
         _monAction[param1] = String(_loc5_.@atctionId);
         _materialToHtidMap[_loc5_.@atctionId] = param1;
         return _loc4_;
      }

      private static function mapGet(param1:*) : XML
      {
         var _loc2_:XML = _map[param1];
         if(_loc2_ == null && _monIdx != null)
         {
            _loc2_ = resolveMonster(String(param1));
         }
         return _loc2_;
      }

      private static function ensureAllMonsters() : void
      {
         var _loc2_:String = null;
         var _loc1_:Array = _monOrder;
         _monOrder = null;
         for each(_loc2_ in _loc1_)
         {
            mapGet(_loc2_);
            if(_monAction[_loc2_] != null)
            {
               _materialToHtidMap[_monAction[_loc2_]] = _loc2_;
            }
         }
      }

      public static function initMonsterDictionaryData(''')
files['game.data.group.HeroDataUtil'] = s

# 5. PrestigeHerosManager: tavern heroes built on first use.
s = rd('game/xmlParsers/pretigeHero/PrestigeHerosManager.as')
s = rep(s, '''      private static var instanceMap:Dictionary;
''', '''      private static var instanceMap:Dictionary;

      private static var _xml:XML;
''')
s = rep(s, '''      public static function initXML(param1:XML) : void
      {''', '''      public static function initXML(param1:XML) : void
      {
         _xml = param1;
         data = null;
      }

      private static function ensure() : void
      {
         if(data == null && _xml != null)
         {
            build(_xml);
         }
      }

      private static function build(param1:XML) : void
      {''')
for fn in ('getHerosByCamp(param1:int, param2:int) : Array', 'getHerosByHeroTempID(param1:int) : RecruitHeroData', 'getHeros() : Array'):
    s = re.sub(r'(public static function %s\n      \{\n(?:         var [^\n]*\n)*)' % re.escape(fn), lambda m: m.group(1) + '         ensure();\n', s, count=1)
assert s.count('ensure();') == 3, s.count('ensure();')
files['game.xmlParsers.pretigeHero.PrestigeHerosManager'] = s

# 6. main: startup reads skill and monster templates as text.
s = rd('main.as')
s = rep(s, '         SkillAndBuffXMLManager.iniSkill(XmlManager.skill);', '''         var _kpRaw:String = XmlManager.getRawXml("skill");
         if(_kpRaw != null)
         {
            SkillAndBuffXMLManager.iniSkillRaw(_kpRaw);
         }
         else
         {
            SkillAndBuffXMLManager.iniSkill(XmlManager.skill);
         }''')
s = rep(s, '         HeroDataUtil.initMonsterDictionaryData(XmlManager.monstersTemplete,XmlManager.monstersData);', '''         var _kpTmpl:String = XmlManager.getRawXml("monster_teml_copyNeedNot");
         var _kpMon:String = XmlManager.getRawXml("monsters");
         if(_kpTmpl != null && _kpMon != null)
         {
            HeroDataUtil.initMonsterDictionaryRaw(_kpTmpl,_kpMon);
         }
         else
         {
            HeroDataUtil.initMonsterDictionaryData(XmlManager.monstersTemplete,XmlManager.monstersData);
         }''')
if TRACE:
    s = trace_lines(s, '      private function initXML() : void', 'KPM')
files['main'] = s

# 7. MonsterXmlLoader: monstersTemplete is never read after startup, so skip appending to it
# (appending would force the 13.6 MB template file to be parsed).
s = rd('mmo/utils/dataLoader/loadTypes/MonsterXmlLoader.as')
s = rep(s, '''            for each(xml in (_data as XML).elements())
            {
               XmlManager.monstersTemplete.appendChild(xml);
            }
''', '')
files['mmo.utils.dataLoader.loadTypes.MonsterXmlLoader'] = s

if TRACE:
    s = rd('game/ui/uiloader/UIloader.as')
    s = trace_lines(s, '      private function onEnterFrameHandler(param1:Event) : void', 'KPT')
    files['game.ui.uiloader.UIloader'] = s

args = []
for cls, body in files.items():
    p = os.path.join(out, cls + '.as')
    open(p, 'w', encoding='utf-8').write(body)
    args += [cls, p]
open(os.path.join(out, 'replace_args.txt'), 'w', encoding='utf-8').write('\n'.join(args))
print('classes', len(files), 'lazy fields', len(lazy))
