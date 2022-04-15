// by wuwbobo2021 <https://github.com/wuwbobo2021>, <wuwbobo@outlook.com>
// If you have found bugs in this program, please pull an issue, or contact me.

#ifndef RAMDOM_PICKER_TABLE_H
#define RAMDOM_PICKER_TABLE_H

#include <stdexcept>
#include <vector>
#include <string>

namespace RandomPicker
{

class Item
{
	std::string m_name;
	double m_value;

public:
	Item(const std::string& name, double value);
	
	const std::string& name() const;
	bool name(const std::string& name);
	
	double value() const;
	bool value(double value);
};

inline Item::Item(const std::string& name, double value)
{
	if (! this->name(name))
		throw std::invalid_argument("RandomPicker::Item::Item(): invalid name.");
	if (! this->value(value))
		throw std::invalid_argument("RandomPicker::Item::Item(): minus value.");
	
	m_value = value;
}

inline const std::string& Item::name() const
{
	return m_name;
}

inline double Item::value() const
{
	return m_value;
}

inline bool Item::value(double value)
{
	if (value < 0) return false;
	m_value = value; return true;
}

class Table
{
	std::vector<Item> m_vect;
	
	int find_name(const std::string& name) const;
	
public:
	bool repetitive_picking = false;
	bool power_inversed = false;
	
	unsigned int count() const;
	bool is_empty() const;
	
	Item& operator[](unsigned int index);
	double item_value(const std::string& name) const;
	void item(Item& item);
	
	void clear();
	void scale(float scaler);
	void inverse();
	bool input(std::istream& ist);
	bool output(std::ostream& ost) const;
	
	bool open(const std::string& path);
	bool save(const std::string& path) const;
};

inline unsigned int Table::count() const
{
	return m_vect.size();
}

inline bool Table::is_empty() const
{
	return this->count() == 0;
}

inline Item& Table::operator[](unsigned int index)
{
	if (index > this->count() - 1)
		throw std::invalid_argument("Table::operator[](): invalid index.");
	return m_vect[index];
}

inline double Table::item_value(const std::string& name) const
{
	int i = this->find_name(name);
	if (i < 0) return 0;
	return m_vect[i].value();
}

inline void Table::item(Item& item)
{
	int i = this->find_name(item.name());
	if (i < 0)
		m_vect.push_back(item);
	else
		m_vect[i].value(item.value());
}

inline void Table::clear()
{
	m_vect.clear();
}

}

#endif

