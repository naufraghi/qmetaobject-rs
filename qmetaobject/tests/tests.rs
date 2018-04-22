extern crate qmetaobject;
use qmetaobject::*;

#[macro_use]
extern crate lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
}

pub fn do_test<T: QObject + Sized>(mut obj: T, qml: &str) -> bool {

    let _lock = TEST_MUTEX.lock().unwrap();

    let qml_text = "import QtQuick 2.0\n".to_owned() + qml;

    let mut engine = QmlEngine::new();
    engine.set_object_property("_obj".into(), &mut obj);
    engine.load_data(qml_text.into());
    engine.invoke_method("doTest".into(), &[]).to_bool()
}


#[test]
fn self_test() {

    #[derive(QObject,Default)]
    struct Basic {
        base: qt_base_class!(trait QObject),
        value: qt_property!(bool),
    }

    let mut obj = Basic::default();
    obj.value = true;
    assert!(do_test(obj, "Item { function doTest() { return _obj.value  } }"));

    let mut obj = Basic::default();
    obj.value = false;
    assert!(!do_test(obj, "Item { function doTest() { return _obj.value  } }"));

}


#[derive(QObject,Default)]
struct MyObject {
    base: qt_base_class!(trait QObject),
    prop_x: qt_property!(u32; NOTIFY prop_x_changed),
    prop_x_changed: qt_signal!(),
    prop_y: qt_property!(String; NOTIFY prop_y_changed),
    prop_y_changed: qt_signal!(),
    prop_z: qt_property!(QString; NOTIFY prop_z_changed),
    prop_z_changed: qt_signal!(),

    multiply_and_add1: qt_method!(fn multiply_and_add1(&self, a: u32, b:u32) -> u32 { a*b + 1 }),

    concatenate_strings: qt_method!(fn concatenate_strings(
            &self, a: QString, b:QString, c: QByteArray) -> QString {
        let res = a.to_string() + &(b.to_string()) + &(c.to_string());
        QString::from(&res as &str)
    })
}


#[test]
fn property_read_write_notify() {

    let obj = MyObject::default();
    assert!(do_test(obj, "Item {
        property int yo: _obj.prop_x;
        function doTest() {
            _obj.prop_x = 123;
            return yo === 123;
        }}"));

    let obj = MyObject::default();
    assert!(do_test(obj, "Item {
        property string yo: _obj.prop_y + ' ' + _obj.prop_z;
        function doTest() {
            _obj.prop_y = 'hello';
            _obj.prop_z = 'world';
            return yo === 'hello world';
        }}"));
}

#[test]
fn call_method() {

    let obj = MyObject::default();
    assert!(do_test(obj, "Item {
        function doTest() {
            return _obj.multiply_and_add1(45, 76) === 45*76+1;
        }}"));

    let obj = MyObject::default();
    assert!(do_test(obj, "Item {
        function doTest() {
            return _obj.concatenate_strings('abc', 'def', 'hij') == 'abcdefhij';
        }}"));

    let obj = MyObject::default();
    assert!(do_test(obj, "Item {
        function doTest() {
            return _obj.concatenate_strings(123, 456, 789) == '123456789';
        }}"));
}



#[test]
fn simple_model() {

    #[derive(Default)]
    struct TM {
        a: QString,
        b: u32,
    }
    impl qmetaobject::listmodel::SimpleListItem for TM {
        fn get(&self, idx : i32) -> QVariant {
            match idx {
                0 => self.a.clone().into(),
                1 => self.b.clone().into(),
                _ => QVariant::default()
            }
        }
        fn names() -> Vec<QByteArray> {
            vec![ QByteArray::from("a"), QByteArray::from("b") ]
        }
    }
    // FIXME! why vec! here?
    let model : qmetaobject::listmodel::SimpleListModel<TM> = (vec![TM{a: "hello".into(), b:1}]).into_iter().collect();
    assert!(do_test(model, "Item {
            Repeater{
                id: rep;
                model:_obj;
                Text {
                    text: a + b;
                }
            }
            function doTest() {
                console.log('simple_model:', rep.count, rep.itemAt(0).text);
                return rep.count === 1 && rep.itemAt(0).text === 'hello1';
            }}"));
}

#[derive(Default, QObject)]
struct RegisteredObj {
    base: qt_base_class!(trait QObject),
    value: qt_property!(u32),
    square: qt_method!(fn square(&self, v : u32) -> u32 { self.value * v } ),

}

#[test]
fn register_type() {
    qml_register_type::<RegisteredObj>("TestRegister", 1, 0, "RegisteredObj");

    let obj = MyObject::default(); // not used but needed for do_test
    assert!(do_test(obj, "import TestRegister 1.0;
        Item {
            RegisteredObj {
                id: test;
                value: 55;
            }
            function doTest() {
                return test.square(66) === 55*66;
            }
        }"));
}
