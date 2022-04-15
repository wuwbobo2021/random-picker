// by wuwbobo2021 <https://github.com/wuwbobo2021>, <wuwbobo@outlook.com>
// If you have found bugs in this program, please pull an issue, or contact me.

#include "picker.h"

#include <iostream>
#include <sstream>

using namespace std;

RandomPicker::Table table;
RandomPicker::Picker picker(table);

void config(string& save_path);
void print_help();
unsigned int read_value(const string& str);
bool ask_yes_no();

int main(int argc, char** argv)
{
	bool arg_help = false, arg_show_table = false, arg_config = false, arg_test = false;
	unsigned int arg_amount = 0; string arg_path = "";
	bool flag_opened = false;
	
	string arg;
	for (unsigned int i = 1; i < argc; i++) {
		arg = argv[i];
		if (arg[0] == '-' && arg.length() > 1) {
			switch (arg[1]) {
				case 'h': arg_help = true; break;
				case 's': arg_show_table = true; break;
				case 'c': arg_config = true; break;
				case 't': arg_test = true; break;
				default: break;
			}
		} else {
			unsigned int val = read_value(arg);
			if (val > 0 && (val <= table.count() || table.repetitive_picking))
				arg_amount = val;
			else {
				arg_path = arg;
				if (table.open(arg_path)) flag_opened = true; 
			}
		}
	}
	
	if (arg_help || arg_path.length() == 0) {print_help(); return 0;}
	
	if (arg_config) {config(arg_path); return 0;}
	else if (!flag_opened || arg_amount == 0) {print_help(); return 0;}
	
	if (arg_show_table) {
		table.output(cout);
		if (table.count() > 0 && arg_amount > 0) {
			RandomPicker::Table cal; picker.calculate(arg_amount, cal);
			cal.scale(100.0);
			cout << "\nAbsolute values (%):\n";
			cal.output(cout);
			if (table.repetitive_picking)
				cout << "Note: Probabilities in this table are for a picking operation of a single item, "
				     << "you can calculate probability of <i>th item in a group of n items by: 1 - (1 - Pi)^m.\n";
		}
		return 0;
	}
	
	if (! arg_test) {
		vector<string> result;
		picker.pick(arg_amount, result);
		for (unsigned int i = 0; i < result.size(); i++)
			cout << result[i] << ' ';
		cout << '\n';
	} else {
		std::random_device ran_dev;
		cout << "entropy() returned by current standard library random_device: "
		     << ran_dev.entropy() << ".\n";
		
		RandomPicker::Table result;
		picker.test(1000000, arg_amount, result);
		
		if (! table.repetitive_picking) {
			result.scale(1.0 / 10000.0);
			cout << "Test result indicating probabilities (%) of occurence in a group of results:\n";
		} else {
			result.scale(1.0 / (10000.0*arg_amount));
			cout << "Test result of frequencies (%):\n";
		}
		result.output(cout);
	}
	
	return 0;
}

void config(string& save_path)
{
	if (! table.is_empty()) {
		cout << "Existing items:\n";
		table.output(cout); cout << '\n';
	}
	
	cout << "Is it allowed to pick items repetitively?";
	table.repetitive_picking = ask_yes_no();
	cout << "Should the power values of items be inversed to calculate their probability? "
	     << "In this case the power value represents its prize, the higher the prize, "
	     << "the lower the probability of being picked up.\n";
	table.power_inversed = ask_yes_no();
	
	cout << "Input items, seperate names and power values with spaces, "
	     << "delete item with `delete <name>`, input end at last:\n";
	if (! table.input(cin))
		cout << "Sorry, part of your input is not recorded. Make sure your names consist of "
		     << "letters, digits, or underline characters, without any space.\n";
	
	cout << "Please check the recorded items below:\n";
	table.output(cout);
	
	if (table.is_empty()) return;
	
	while (! table.save(save_path)) {
		cout << "Sorry, failed to save file \"" << save_path << "\".\n"
		     << "Enter file path: ";
		save_path = ""; cin >> save_path;
	}
}

void print_help()
{
	cout << "random-picker <table_file> <amount>\n"
	     << "Options:\n"
	     << "-h\t\t\tShow this help\n"
	     << "-c <file>\t\tDo configuration and save table file\n"
	     << "-s <file> [amount]\tPrint current table, show table of absolute values if amount is given\n"
	     << "-t <file> <amount>\tTest the random engine by statistics of 1,000,000 groups of results\n"
		 << "Note: When repetitive mode is off, <amount> must not exceed amount of items in the table.\n";
}

unsigned int read_value(const string& str)
{
	static stringstream sst;
	sst.clear(); sst.str(str);
	
	unsigned int val = 0;
	sst >> val; return val;
}

bool ask_yes_no()
{
	static char buf[4096];
	cout << " (Y/n) ";
	cin.getline(buf, 4096);
	return buf[0] == 'Y' || buf[0] == 'y';
}

