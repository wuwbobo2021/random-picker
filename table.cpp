// by wuwbobo2021 <https://github.com/wuwbobo2021>, <wuwbobo@outlook.com>
// If you have found bugs in this program, please pull an issue, or contact me.

#include "table.h"

#include <cctype>
#include <fstream>

using namespace RandomPicker;

const std::string String_Repetitive_Picking = "repetitive_picking";
const std::string String_Power_Inversed = "power_inversed";

const std::string String_Delete = "delete";
const std::string String_End_Of_Input = "end";

bool Item::name(const std::string& name)
{
	if (name.length() == 0) return false;
	
	const char* str = name.c_str(); unsigned int l = name.length();
	for (unsigned int i = 0; i < l; i++)
		if (str[i] != '_' && !isalpha(str[i]) && !isdigit(str[i]))
			return false;
	
	m_name = name; return true;
}

int Table::find_name(const std::string& name) const
{
	for (unsigned int i = 0; i < this->count(); i++)
		if (m_vect[i].name() == name) return i;
	
	return -1;
}

void Table::scale(float scaler)
{
	for (unsigned int i = 0; i < this->count(); i++)
		m_vect[i].value(scaler * m_vect[i].value());
}

void Table::inverse()
{
	for (unsigned int i = 0; i < this->count(); i++)
		if (m_vect[i].value() > 0)
			m_vect[i].value(1.0 / m_vect[i].value());
	
	this->power_inversed = ! this->power_inversed;
}

bool Table::input(std::istream& ist)
{
	std::string name; double val; Item it("none", 0);
	while (! ist.eof()) {
		ist >> name;
		if (! ist) return true;
		
		if (name == String_End_Of_Input) return true;
		else if (name == String_Delete) {
			ist >> name; if (! ist) return true;
			unsigned int i = this->find_name(name);
			if (i < 0) continue;
			m_vect.erase(m_vect.begin() + i); continue;
		}
		else if (name == String_Repetitive_Picking) {
			this->repetitive_picking = true; continue;
		} else if (name == String_Power_Inversed) {
			this->power_inversed = true; continue;
		}
		
		ist >> val;
		if (! ist) {ist.clear(); return false;}
		
		try {
			it = Item(name, val);
			this->item(it);
		} catch (std::invalid_argument ex) {
			return false;
		}
	}
	
	return true;
}

bool Table::output(std::ostream& ost) const
{
	if (this->is_empty()) return false;
	
	if (this->repetitive_picking)
		ost << String_Repetitive_Picking << '\n';
	if (this->power_inversed)
		ost << String_Power_Inversed << '\n';
	
	unsigned int cnt = this->count();
	for (unsigned int i = 0; i < cnt; i++)
		ost << m_vect[i].name() << "\t\t" << m_vect[i].value() << '\n';
	
	return true;
}

bool Table::open(const std::string& path)
{
	std::ifstream ifs(path, std::ios_base::in);
	if (! ifs.is_open()) return false;
	return this->input(ifs);
}

bool Table::save(const std::string& path) const
{
	if (this->is_empty()) return false;
	std::ofstream ofs(path, std::ios_base::out);
	if (! ofs.is_open()) return false;
	return this->output(ofs);
}

