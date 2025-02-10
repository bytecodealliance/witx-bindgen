package wit.exports.test.lists;

import static wit.worlds.ListsImpl.expect;

import java.util.ArrayList;

import wit.worlds.Lists.Tuple2;
import wit.worlds.Lists.Tuple3;

public class TestImpl {
    public static void emptyListParam(byte[] a) {
        expect(a.length == 0);
    }

    public static void emptyStringParam(String a) {
        expect(a.length() == 0);
    }

    public static byte[] emptyListResult() {
        return new byte[0];
    }

    public static String emptyStringResult() {
        return "";
    }

    public static void listParam(byte[] a) {
        expect(a.length == 4);
        expect(a[0] == 1);
        expect(a[1] == 2);
        expect(a[2] == 3);
        expect(a[3] == 4);
    }

    public static void listParam2(String a) {
        expect(a.equals("foo"));
    }

    public static void listParam3(ArrayList<String> a) {
        expect(a.size() == 3);
        expect(a.get(0).equals("foo"));
        expect(a.get(1).equals("bar"));
        expect(a.get(2).equals("baz"));
    }

    public static void listParam4(ArrayList<ArrayList<String>> a) {
        expect(a.size() == 2);
        expect(a.get(0).size() == 2);
        expect(a.get(1).size() == 1);

        expect(a.get(0).get(0).equals("foo"));
        expect(a.get(0).get(1).equals("bar"));
        expect(a.get(1).get(0).equals("baz"));
    }

    public static void listParam5(ArrayList<Tuple3<Byte, Integer, Byte>> a) {
        expect(a.size() == 2);
        expect(a.get(0).f0 == 1);
        expect(a.get(0).f1 == 2);
        expect(a.get(0).f2 == 3);
        expect(a.get(1).f0 == 4);
        expect(a.get(1).f1 == 5);
        expect(a.get(1).f2 == 6);
    }

    public static void listParamLarge(ArrayList<String> a) {
        expect(a.size() == 1000);
    }

    public static byte[] listResult() {
        return new byte[] { (byte) 1, (byte) 2, (byte) 3, (byte) 4, (byte) 5 };
    }

    public static String listResult2() {
        return "hello!";
    }

    public static ArrayList<String> listResult3() {
        return new ArrayList<String>() {{
            add("hello,");
            add("world!");
        }};
    }

    public static byte[] listRoundtrip(byte[] a) {
        return a;
    }

    public static String stringRoundtrip(String a) {
        return a;
    }

    public static Tuple2<byte[], byte[]> listMinmax8(byte[] a, byte[] b) {
        return new Tuple2<>(a, b);
    }

    public static Tuple2<short[], short[]> listMinmax16(short[] a, short[] b) {
        return new Tuple2<>(a, b);
    }

    public static Tuple2<int[], int[]> listMinmax32(int[] a, int[] b) {
        return new Tuple2<>(a, b);
    }

    public static Tuple2<long[], long[]> listMinmax64(long[] a, long[] b) {
        return new Tuple2<>(a, b);
    }

    public static Tuple2<float[], double[]> listMinmaxFloat(float[] a, double[] b) {
        return new Tuple2<>(a, b);
    }
}
